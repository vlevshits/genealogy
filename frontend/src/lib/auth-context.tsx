import React, {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
} from 'react';
import {
  fetchCurrentUser,
  fetchSetupStatus,
  logoutSession,
  setAuthToken,
  SetupStatusResponse,
  UserDto,
} from '../api/client';

export interface AuthContextValue {
  user: UserDto | null;
  setupStatus: SetupStatusResponse | null;
  isLoading: boolean;
  isAuthenticated: boolean;
  backendReachable: boolean;
  promptPasskeyEnrollment: boolean;
  setPromptPasskeyEnrollment: (val: boolean) => void;
  establishSession: (
    user: UserDto,
    sessionToken?: string,
    promptPasskey?: boolean
  ) => void;
  updateUser: (user: UserDto) => void;
  refreshAuth: () => Promise<void>;
  logout: () => Promise<void>;
}

const DEFAULT_SETUP_STATUS: SetupStatusResponse = {
  is_initialized: true,
  tree_title: 'Levshits Family Archive',
  default_locale: 'en',
  enable_public_access: false,
  auth_password_enabled: true,
  smtp_enabled: false,
  rp_id: 'localhost',
  rp_name: 'Levshits Family Archive',
};

const AuthContext = createContext<AuthContextValue>({
  user: null,
  setupStatus: DEFAULT_SETUP_STATUS,
  isLoading: false,
  isAuthenticated: false,
  backendReachable: false,
  promptPasskeyEnrollment: false,
  setPromptPasskeyEnrollment: () => {},
  establishSession: () => {},
  updateUser: () => {},
  refreshAuth: async () => {},
  logout: async () => {},
});

export interface AuthProviderProps {
  initialUser?: UserDto | null;
  initialSetupStatus?: SetupStatusResponse | null;
  children: React.ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({
  initialUser = null,
  initialSetupStatus = null,
  children,
}) => {
  const [user, setUser] = useState<UserDto | null>(initialUser);
  const [setupStatus, setSetupStatus] = useState<SetupStatusResponse | null>(
    initialSetupStatus
  );
  const [isLoading, setIsLoading] = useState<boolean>(
    initialUser === null && initialSetupStatus === null
  );
  const [backendReachable, setBackendReachable] = useState<boolean>(
    initialSetupStatus !== null
  );
  const [promptPasskeyEnrollment, setPromptPasskeyEnrollment] =
    useState<boolean>(false);

  const refreshAuth = useCallback(async () => {
    try {
      const status = await fetchSetupStatus();
      setSetupStatus(status);
      setBackendReachable(true);
      if (status.is_initialized) {
        try {
          const me = await fetchCurrentUser();
          setUser(me);
        } catch {
          setUser(null);
        }
      } else {
        setUser(null);
      }
    } catch {
      // Backend offline fallback
      setBackendReachable(false);
      setSetupStatus((prev) => prev ?? DEFAULT_SETUP_STATUS);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    if (initialUser !== null || initialSetupStatus !== null) {
      setIsLoading(false);
      return;
    }
    void refreshAuth();
  }, [initialUser, initialSetupStatus, refreshAuth]);

  const establishSession = useCallback(
    (
      nextUser: UserDto,
      sessionToken?: string,
      promptPasskey = false
    ) => {
      if (sessionToken) {
        setAuthToken(sessionToken);
      }
      setUser(nextUser);
      setPromptPasskeyEnrollment(promptPasskey);
      setSetupStatus((prev) =>
        prev ? { ...prev, is_initialized: true } : { ...DEFAULT_SETUP_STATUS, is_initialized: true }
      );
    },
    []
  );

  const updateUser = useCallback((nextUser: UserDto) => {
    setUser(nextUser);
  }, []);

  const logout = useCallback(async () => {
    try {
      await logoutSession();
    } catch {
      // Ignore network error on logout
    }
    setAuthToken(null);
    setUser(null);
    setPromptPasskeyEnrollment(false);
  }, []);

  return (
    <AuthContext.Provider
      value={{
        user,
        setupStatus,
        isLoading,
        isAuthenticated: Boolean(user),
        backendReachable,
        promptPasskeyEnrollment,
        setPromptPasskeyEnrollment,
        establishSession,
        updateUser,
        refreshAuth,
        logout,
      }}
    >
      {children}
    </AuthContext.Provider>
  );
};

export function useAuth(): AuthContextValue {
  return useContext(AuthContext);
}
