import { XIcon } from "@phosphor-icons/react";
import type { KeyboardEvent } from "react";
import { useEffect, useState } from "react";

import { Badge, Input } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { PROJECT_MAX_TOPIC_LENGTH, PROJECT_MAX_TOPICS } from "@flow/types";

type Props = {
  id?: string;
  value: string[];
  onChange: (topics: string[]) => void;
  // Told whether the text not yet added holds a topic too long to add, so
  // the form can wait until it is shortened or cleared.
  onDraftInvalidChange?: (invalid: boolean) => void;
};

const isTooLong = (topic: string) =>
  [...topic.trim()].length > PROJECT_MAX_TOPIC_LENGTH;

/**
 * Adds a topic on Enter, comma or blur. Topics are trimmed and blanks and
 * repeats are dropped, as the server does; one that is too long stays in the
 * input to be shortened.
 */
const TopicsInput: React.FC<Props> = ({
  id,
  value,
  onChange,
  onDraftInvalidChange,
}) => {
  const t = useT();
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | undefined>(undefined);

  const draftInvalid = draft.split(",").some(isTooLong);
  useEffect(() => {
    onDraftInvalidChange?.(draftInvalid);
    // Text that is no longer shown cannot hold the form back.
    return () => onDraftInvalidChange?.(false);
  }, [draftInvalid, onDraftInvalidChange]);

  const addTopics = (raw: string) => {
    const candidates = raw
      .split(",")
      .map((topic) => topic.trim())
      .filter((topic) => topic !== "");
    if (!candidates.length) {
      setDraft("");
      return;
    }
    if (candidates.some(isTooLong)) {
      setError(
        t("Each topic can be at most {{max}} characters.", {
          max: PROJECT_MAX_TOPIC_LENGTH,
        }),
      );
      return;
    }

    const next = [...value];
    let overLimit = false;
    for (const topic of candidates) {
      if (next.includes(topic)) continue;
      if (next.length >= PROJECT_MAX_TOPICS) {
        overLimit = true;
        break;
      }
      next.push(topic);
    }
    setError(
      overLimit
        ? t("A project can have at most {{max}} topics.", {
            max: PROJECT_MAX_TOPICS,
          })
        : undefined,
    );
    if (next.length !== value.length) onChange(next);
    setDraft("");
  };

  const removeTopic = (topic: string) => {
    setError(undefined);
    onChange(value.filter((v) => v !== topic));
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLInputElement>) => {
    if (e.nativeEvent.isComposing) return;
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      addTopics(draft);
    } else if (e.key === "Backspace" && draft === "" && value.length) {
      removeTopic(value[value.length - 1]);
    }
  };

  return (
    <div className="flex flex-col gap-2">
      {value.length > 0 && (
        <div className="flex flex-wrap gap-1">
          {value.map((topic) => (
            <Badge
              key={topic}
              variant="secondary"
              className="gap-1 border-accent font-normal">
              <span className="max-w-[240px] truncate">{topic}</span>
              <button
                type="button"
                className="rounded-full opacity-70 hover:opacity-100"
                aria-label={t("Remove topic {{topic}}", { topic })}
                onClick={() => removeTopic(topic)}>
                <XIcon className="size-3" />
              </button>
            </Badge>
          ))}
        </div>
      )}
      <Input
        id={id}
        value={draft}
        placeholder={t("Add a topic and press Enter...")}
        onChange={(e) => {
          setError(undefined);
          setDraft(e.target.value);
        }}
        onKeyDown={handleKeyDown}
        onBlur={() => draft.trim() && addTopics(draft)}
      />
      {error && <p className="text-xs text-destructive">{error}</p>}
    </div>
  );
};

export { TopicsInput };
