import { enUS, ja, es, fr, zhCN, Locale } from "date-fns/locale";
import { useState } from "react";
import DatePicker, { registerLocale } from "react-datepicker";

import "react-datepicker/dist/react-datepicker.css";
import "./styles.css";
import { AvailableLanguage, availableLanguages, useLang } from "@flow/lib/i18n";

type Props = {
  className?: string;
};

export const locales: Record<AvailableLanguage, Locale> = {
  en: enUS,
  ja,
  es,
  fr,
  zh: zhCN,
};

const DateTimePicker: React.FC<Props> = ({ className }) => {
  const [startDate, setStartDate] = useState(new Date());

  const currentLang = useLang();
  if (currentLang in availableLanguages) {
    registerLocale(currentLang, locales[currentLang as AvailableLanguage]);
  }

  return (
    <DatePicker
      className={`w-full rounded border bg-transparent px-3 py-2 text-sm font-extralight focus-visible:border-none ${className}`}
      selected={startDate}
      dateFormat="yyyy/MM/dd HH:mm"
      timeFormat="HH:mm"
      timeIntervals={15}
      showTimeSelect
      popperPlacement="right"
      locale="ja"
      showPopperArrow={false}
      onChange={(date) => date && setStartDate(date)}
    />
  );
};

export default DateTimePicker;
