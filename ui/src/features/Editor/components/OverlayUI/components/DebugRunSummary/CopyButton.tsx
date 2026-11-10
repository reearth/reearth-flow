import { CheckIcon, CopyIcon } from "@phosphor-icons/react";
import { useCallback, useEffect, useState } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";

// A small copy button that shows a check for a moment once the text is copied.
const CopyButton: React.FC<{ text: string; className?: string }> = ({
  text,
  className,
}) => {
  const t = useT();
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const timeout = setTimeout(() => setCopied(false), 1500);
    return () => clearTimeout(timeout);
  }, [copied]);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
    } catch {
      // Nothing to recover: the text stays on screen to select by hand.
    }
  }, [text]);

  return (
    <IconButton
      className={className}
      tooltipText={copied ? t("Copied") : t("Copy")}
      icon={
        copied ? (
          <CheckIcon weight="light" size={14} />
        ) : (
          <CopyIcon weight="light" size={14} />
        )
      }
      onClick={handleCopy}
    />
  );
};

export default CopyButton;
