// =============================================================================
// Story Extraction Engine Configuration
// Extraction and transcription are executed server-side via GEMINI_API_KEY in .env.
// No API keys are ever stored or exposed in the client UI.
// =============================================================================

export const STORY_EXTRACTION_ENGINE_STORAGE_KEY =
  'genealogy_story_extraction_engine';

export type StoryExtractionEngine = 'gemini' | 'builtin';

// Purge any legacy API key from localStorage if it was previously set
if (typeof window !== 'undefined' && window.localStorage) {
  try {
    localStorage.removeItem('genealogy_gemini_api_key');
  } catch {
    // Non-blocking
  }
}

export function getExtractionEngine(): StoryExtractionEngine {
  if (typeof window === 'undefined' || !window.localStorage) {
    return 'gemini';
  }
  const stored = localStorage.getItem(STORY_EXTRACTION_ENGINE_STORAGE_KEY);
  if (stored === 'gemini' || stored === 'builtin') {
    return stored;
  }
  return 'gemini';
}

export function setExtractionEngine(engine: StoryExtractionEngine): void {
  if (typeof window === 'undefined' || !window.localStorage) {
    return;
  }
  localStorage.setItem(STORY_EXTRACTION_ENGINE_STORAGE_KEY, engine);
}
