import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import React from 'react';
import { renderToStaticMarkup } from 'react-dom/server';
import {
  apiFetch,
  ApiError,
  getAuthToken,
  setAuthToken,
} from '../../src/api/core';
import * as authApiModule from '../../src/api/auth-api';
import { AuthProvider, useAuth, type AuthContextValue } from '../../src/lib/auth-context';

describe('auth-token-resilience (STAB-05 / defensive token management)', () => {
  const originalLocalStorage = globalThis.localStorage;
  let mockStore: Record<string, string> = {};

  beforeEach(() => {
    mockStore = {};
    const mockLocalStorage = {
      getItem: vi.fn((key: string) => mockStore[key] ?? null),
      setItem: vi.fn((key: string, value: string) => {
        mockStore[key] = String(value);
      }),
      removeItem: vi.fn((key: string) => {
        delete mockStore[key];
      }),
      clear: vi.fn(() => {
        mockStore = {};
      }),
      length: 0,
      key: vi.fn((index: number) => Object.keys(mockStore)[index] ?? null),
    };

    Object.defineProperty(globalThis, 'localStorage', {
      value: mockLocalStorage,
      configurable: true,
      writable: true,
    });

    setAuthToken(null);
  });

  afterEach(() => {
    vi.restoreAllMocks();
    setAuthToken(null);
    Object.defineProperty(globalThis, 'localStorage', {
      value: originalLocalStorage,
      configurable: true,
      writable: true,
    });
  });

  describe('getAuthToken & setAuthToken defensive sanitization', () => {
    it('returns null and cleans localStorage when stored value is the string "null"', () => {
      mockStore['genealogy_session_token'] = 'null';
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });

    it('returns null and cleans localStorage when stored value is the string "undefined"', () => {
      mockStore['genealogy_session_token'] = 'undefined';
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });

    it('returns null and cleans localStorage when stored value is empty or pure whitespace', () => {
      mockStore['genealogy_session_token'] = '   ';
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });

    it('sanitizes when setAuthToken is called with "null" or "undefined" or whitespace', () => {
      setAuthToken('   ');
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();

      setAuthToken('null');
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();

      setAuthToken('undefined');
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });

    it('stores and returns valid trimmed token strings', () => {
      setAuthToken('  valid-session-jwt-12345  ');
      expect(getAuthToken()).toBe('valid-session-jwt-12345');
      expect(mockStore['genealogy_session_token']).toBe('valid-session-jwt-12345');
    });

    it('clears token in memory and localStorage when setAuthToken(null) is called', () => {
      setAuthToken('valid-session-jwt-12345');
      expect(getAuthToken()).toBe('valid-session-jwt-12345');

      setAuthToken(null);
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });
  });

  describe('apiFetch automatic token purge on 401 / session_expired', () => {
    it('purges token from memory and localStorage on 401 Unauthorized', async () => {
      setAuthToken('expired-session-jwt');
      expect(getAuthToken()).toBe('expired-session-jwt');

      vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            error: {
              code: 'unauthorized',
              message: 'Authentication required',
            },
          }),
          { status: 401, headers: { 'Content-Type': 'application/json' } }
        )
      );

      await expect(apiFetch('/auth/me')).rejects.toThrow(ApiError);
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });

    it('purges token on not_authenticated error code regardless of status', async () => {
      setAuthToken('revoked-session-jwt');
      expect(getAuthToken()).toBe('revoked-session-jwt');

      vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            error: {
              code: 'not_authenticated',
              message: 'Session has ended',
            },
          }),
          { status: 403, headers: { 'Content-Type': 'application/json' } }
        )
      );

      await expect(apiFetch('/auth/me')).rejects.toThrow(ApiError);
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });

    it('purges token on session_expired error code', async () => {
      setAuthToken('timed-out-jwt');
      expect(getAuthToken()).toBe('timed-out-jwt');

      vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            error: {
              code: 'session_expired',
              message: 'Your session has expired',
            },
          }),
          { status: 403, headers: { 'Content-Type': 'application/json' } }
        )
      );

      await expect(apiFetch('/persons')).rejects.toThrow(ApiError);
      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });

    it('retains token on other HTTP errors like 404 or 500', async () => {
      setAuthToken('active-token-xyz');

      vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
        new Response(
          JSON.stringify({
            error: {
              code: 'not_found',
              message: 'Record not found',
            },
          }),
          { status: 404, headers: { 'Content-Type': 'application/json' } }
        )
      );

      await expect(apiFetch('/persons/non-existent')).rejects.toThrow(ApiError);
      expect(getAuthToken()).toBe('active-token-xyz');
    });

    it('attaches Authorization: Bearer token header when token is present', async () => {
      setAuthToken('valid-token-abc');

      const fetchSpy = vi.spyOn(globalThis, 'fetch').mockResolvedValueOnce(
        new Response(JSON.stringify({ ok: true }), {
          status: 200,
          headers: { 'Content-Type': 'application/json' },
        })
      );

      await apiFetch<{ ok: boolean }>('/auth/me');

      expect(fetchSpy).toHaveBeenCalledTimes(1);
      const requestHeaders = fetchSpy.mock.calls[0][1]?.headers as Headers;
      expect(requestHeaders.get('Authorization')).toBe('Bearer valid-token-abc');
      expect(requestHeaders.get('X-Requested-With')).toBe('genealogy');
    });
  });

  describe('AuthProvider refreshAuth initial auth check & token purge', () => {
    it('purges token when fetchCurrentUser rejects in refreshAuth', async () => {
      setAuthToken('stale-jwt-from-old-session');
      expect(getAuthToken()).toBe('stale-jwt-from-old-session');

      vi.spyOn(authApiModule, 'fetchSetupStatus').mockResolvedValue({
        is_initialized: true,
        tree_title: 'Family Tree',
        default_locale: 'en',
        enable_public_access: false,
        auth_password_enabled: true,
        smtp_enabled: false,
        rp_id: 'localhost',
        rp_name: 'Family Tree',
      });

      vi.spyOn(authApiModule, 'fetchCurrentUser').mockRejectedValue(
        new ApiError(401, {
          code: 'not_authenticated',
          message: 'Session has expired',
        })
      );

      let capturedAuth: AuthContextValue | null = null;
      function Consumer() {
        capturedAuth = useAuth();
        return null;
      }

      renderToStaticMarkup(
        React.createElement(AuthProvider, null, React.createElement(Consumer))
      );

      expect(capturedAuth).not.toBeNull();
      await (capturedAuth as unknown as AuthContextValue).refreshAuth();

      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });

    it('purges token when instance is not yet initialized', async () => {
      setAuthToken('leftover-token');
      expect(getAuthToken()).toBe('leftover-token');

      vi.spyOn(authApiModule, 'fetchSetupStatus').mockResolvedValue({
        is_initialized: false,
        tree_title: 'Family Tree',
        default_locale: 'en',
        enable_public_access: false,
        auth_password_enabled: true,
        smtp_enabled: false,
        rp_id: 'localhost',
        rp_name: 'Family Tree',
      });

      let capturedAuth: AuthContextValue | null = null;
      function Consumer() {
        capturedAuth = useAuth();
        return null;
      }

      renderToStaticMarkup(
        React.createElement(AuthProvider, null, React.createElement(Consumer))
      );

      expect(capturedAuth).not.toBeNull();
      await (capturedAuth as unknown as AuthContextValue).refreshAuth();

      expect(getAuthToken()).toBeNull();
      expect(mockStore['genealogy_session_token']).toBeUndefined();
    });
  });
});
