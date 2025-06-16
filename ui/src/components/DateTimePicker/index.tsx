import { CalendarIcon } from "@phosphor-icons/react";
import { enUS, ja, es, fr, zhCN, Locale } from "date-fns/locale";
import { useEffect, useState } from "react";
import DatePicker, { registerLocale } from "react-datepicker";

import {
  AvailableLanguage,
  availableLanguages,
  useLang,
  useT,
} from "@flow/lib/i18n";

import "react-datepicker/dist/react-datepicker.css";
import "./styles.css";

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
      className={`w-full rounded-md border bg-transparent text-sm focus-visible:border-none dark:font-extralight ${className}`}
      selected={startDate}
      dateFormat="yyyy-MM-dd HH:mm"
      timeFormat="HH:mm"
      timeIntervals={15}
      timeCaption={t("Time")}
      popperPlacement="bottom-start"
      shouldCloseOnSelect={false}
      showTimeSelect
      showIcon
      toggleCalendarOnIconClick
      icon={<CalendarIcon />}
      locale={currentLang}
      showPopperArrow={false}
      onChange={(date) => date && setStartDate(date)}
    />
  );
};

export default DateTimePicker;
