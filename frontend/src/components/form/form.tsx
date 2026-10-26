import React, { createContext, useContext, useState, useCallback } from 'react';
import { type ZodTypeAny } from 'zod';
import { cn } from '../../lib/utils';

export interface UseFormOptions<T extends Record<string, unknown>> {
  defaultValues: T;
  schema?: ZodTypeAny;
  initialWarnings?: Partial<Record<keyof T & string, string>>;
}

export interface FormController<T extends Record<string, unknown>> {
  values: T;
  errors: Record<string, string>;
  warnings: Record<string, string>;
  isDirty: boolean;
  isSubmitting: boolean;
  setValue: <K extends keyof T & string>(name: K, value: T[K]) => void;
  setFieldError: (name: string, message: string) => void;
  setServerErrors: (fieldErrors: Record<string, string>) => void;
  setFieldWarning: (name: string, warning: string | null) => void;
  reset: (nextValues?: T) => void;
  validate: () => boolean;
}

export function useForm<T extends Record<string, unknown>>({
  defaultValues,
  schema,
  initialWarnings = {},
}: UseFormOptions<T>): FormController<T> {
  const [initial, setInitial] = useState<T>(defaultValues);
  const [values, setValues] = useState<T>(defaultValues);
  const [errors, setErrors] = useState<Record<string, string>>({});
  const [warnings, setWarnings] = useState<Record<string, string>>(
    initialWarnings as Record<string, string>
  );
  const [isDirty, setIsDirty] = useState(false);

  const setValue = useCallback(<K extends keyof T & string>(name: K, value: T[K]) => {
    setValues((prev) => {
      const next = { ...prev, [name]: value };
      setIsDirty(JSON.stringify(next) !== JSON.stringify(initial));
      return next;
    });
    setErrors((prev) => {
      if (!(name in prev)) return prev;
      const copy = { ...prev };
      delete copy[name];
      return copy;
    });
  }, [initial]);

  const setFieldError = useCallback((name: string, message: string) => {
    setErrors((prev) => ({ ...prev, [name]: message }));
  }, []);

  const setServerErrors = useCallback((fieldErrors: Record<string, string>) => {
    setErrors((prev) => ({ ...prev, ...fieldErrors }));
  }, []);

  const setFieldWarning = useCallback((name: string, warning: string | null) => {
    setWarnings((prev) => {
      const copy = { ...prev };
      if (warning) {
        copy[name] = warning;
      } else {
        delete copy[name];
      }
      return copy;
    });
  }, []);

  const reset = useCallback(
    (nextValues?: T) => {
      const base = nextValues ?? initial;
      if (nextValues) setInitial(nextValues);
      setValues(base);
      setErrors({});
      setIsDirty(false);
    },
    [initial]
  );

  const validate = useCallback((): boolean => {
    if (!schema) return true;
    const res = schema.safeParse(values);
    if (res.success) {
      setErrors({});
      return true;
    }
    const nextErrs: Record<string, string> = {};
    for (const issue of res.error.issues) {
      const key = issue.path.join('.');
      if (key && !nextErrs[key]) {
        nextErrs[key] = issue.message;
      }
    }
    setErrors(nextErrs);
    return false;
  }, [schema, values]);

  return {
    values,
    errors,
    warnings,
    isDirty,
    isSubmitting: false,
    setValue,
    setFieldError,
    setServerErrors,
    setFieldWarning,
    reset,
    validate,
  };
}

const FormContext = createContext<FormController<Record<string, unknown>> | null>(null);

export function useFormContext<
  T extends Record<string, unknown> = Record<string, unknown>
>(): FormController<T> | null {
  return useContext(FormContext) as FormController<T> | null;
}

export interface FormProps<T extends Record<string, unknown>>
  extends Omit<React.FormHTMLAttributes<HTMLFormElement>, 'onSubmit'> {
  form: FormController<T>;
  onSubmit: (values: T) => void | Promise<void>;
}

export function Form<T extends Record<string, unknown>>({
  form,
  onSubmit,
  className,
  children,
  ...props
}: FormProps<T>) {
  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!form.validate()) return;
    await onSubmit(form.values);
  };

  return (
    <FormContext.Provider value={form as unknown as FormController<Record<string, unknown>>}>
      <form noValidate onSubmit={handleSubmit} className={cn('space-y-5', className)} {...props}>
        {children}
      </form>
    </FormContext.Provider>
  );
}
