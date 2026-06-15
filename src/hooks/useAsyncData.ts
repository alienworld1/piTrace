import { useCallback, useEffect, useState } from "react";

interface AsyncDataState<T> {
  data: T | undefined;
  error: string | undefined;
  isLoading: boolean;
  reload: () => Promise<void>;
}

export function useAsyncData<T>(load: () => Promise<T>, dependencies: unknown[]): AsyncDataState<T> {
  const [data, setData] = useState<T>();
  const [error, setError] = useState<string>();
  const [isLoading, setIsLoading] = useState(true);

  const reload = useCallback(async () => {
    setIsLoading(true);
    setError(undefined);

    try {
      setData(await load());
    } catch (error) {
      setError(error instanceof Error ? error.message : String(error));
    } finally {
      setIsLoading(false);
    }
  }, dependencies);

  useEffect(() => {
    void reload();
  }, [reload]);

  return { data, error, isLoading, reload };
}
