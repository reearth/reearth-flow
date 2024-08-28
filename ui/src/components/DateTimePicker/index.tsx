import { enUS, ja, es, fr, zhCN, Locale } from "date-fns/locale";
import { useEffect, useState } from "react";
import DatePicker, { registerLocale } from "react-datepicker";

import "react-datepicker/dist/react-datepicker.css";
import "./styles.css";
import {
  AvailableLanguage,
  availableLanguages,
  useLang,
  useT,
} from "@flow/lib/i18n";

type Props = {
  className?: string;
};

export const locales: Record<AvailableLanguage, Locale> = {
  en: enUS,
  ja: ja,
  es: es,
  fr: fr,
  zh: zhCN,
};

const DateTimePicker: React.FC<Props> = ({ className }) => {
  const t = useT();
  const [startDate, setStartDate] = useState(new Date());

  const currentLang = useLang();

  useEffect(() => {
    if (currentLang in availableLanguages) {
      registerLocale(currentLang, locales[currentLang as AvailableLanguage]);
    }
  }, [currentLang]);

  return (
    <DatePicker
      className={`w-full rounded-md border bg-transparent px-3 py-2 text-sm font-extralight focus-visible:border-none ${className}`}
      selected={startDate}
      dateFormat="yyyy-MM-dd HH:mm"
      timeFormat="HH:mm"
      timeIntervals={15}
      showTwoColumnMonthYearPicker
      timeCaption={t("Time")}
      showTimeSelect
      popperPlacement="right"
      locale={currentLang}
      showPopperArrow={false}
      onChange={(date) => date && setStartDate(date)}
    />
  );
};

export default DateTimePicker;
