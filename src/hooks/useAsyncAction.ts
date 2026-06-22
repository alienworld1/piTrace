import { useCallback, useRef, useState } from "react";
import { toErrorMessage } from "../utils/errors";

export function useAsyncAction() {
  const [activeKey, setActiveKey] = useState<string>();
  const [error, setError] = useState<string>();
  const activeKeyRef = useRef<string | undefined>(undefined);

  const run = useCallback(async (key: string, action: () => Promise<void>) => {
    if (activeKeyRef.current !== undefined) {
      return false;
    }
    activeKeyRef.current = key;
    setActiveKey(key);
    setError(undefined);
    try {
      await action();
      return true;
    } catch (error) {
      setError(toErrorMessage(error));
      return false;
    } finally {
      activeKeyRef.current = undefined;
      setActiveKey(undefined);
    }
  }, []);

  return { activeKey, error, isRunning: activeKey !== undefined, run };
}
