export const formatTimestamp = (input: string | Date) => {
  const date = new Date(input);
  const pad = (num: number) => (num < 10 ? `0${num}` : num);

  const year = date.getFullYear();
  const month = pad(date.getMonth() + 1);
  const day = pad(date.getDate());
  const hours = pad(date.getHours());
  const minutes = pad(date.getMinutes());
  const seconds = pad(date.getSeconds());

  return `${year}-${month}-${day} ${hours}:${minutes}:${seconds}`;
};
