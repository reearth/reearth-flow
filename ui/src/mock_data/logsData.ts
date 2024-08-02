const rawText = `Timestamp,Status,Transformer,Message
2022-01-01 08:00:00,INFO,DataTransformer,Data processing started
2022-01-01 08:05:00,ERROR,DataTransformer,Error processing data
2022-01-01 08:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-01 08:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-02 09:00:00,INFO,DataTransformer,Data processing started
2022-01-02 09:05:00,ERROR,DataTransformer,Error processing data
2022-01-02 09:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-02 09:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-03 10:00:00,INFO,DataTransformer,Data processing started
2022-01-03 10:05:00,ERROR,DataTransformer,Error processing data
2022-01-03 10:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-03 10:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-04 11:00:00,INFO,DataTransformer,Data processing started
2022-01-04 11:05:00,ERROR,DataTransformer,Error processing data
2022-01-04 11:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-04 11:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-05 12:00:00,INFO,DataTransformer,Data processing started
2022-01-05 12:05:00,ERROR,DataTransformer,Error processing data
2022-01-05 12:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-05 12:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-06 13:00:00,INFO,DataTransformer,Data processing started
2022-01-06 13:05:00,ERROR,DataTransformer,Error processing data
2022-01-06 13:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-06 13:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-07 14:00:00,INFO,DataTransformer,Data processing started
2022-01-07 14:05:00,ERROR,DataTransformer,Error processing data
2022-01-07 14:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-07 14:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-08 15:00:00,INFO,DataTransformer,Data processing started
2022-01-08 15:05:00,ERROR,DataTransformer,Error processing data
2022-01-08 15:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-08 15:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-09 16:00:00,INFO,DataTransformer,Data processing started
2022-01-09 16:05:00,ERROR,DataTransformer,Error processing data
2022-01-09 16:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-09 16:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-10 17:00:00,INFO,DataTransformer,Data processing started
2022-01-10 17:05:00,ERROR,DataTransformer,Error processing data
2022-01-10 17:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-10 17:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-11 18:00:00,INFO,DataTransformer,Data processing started
2022-01-11 18:05:00,ERROR,DataTransformer,Error processing data
2022-01-11 18:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-11 18:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-12 19:00:00,INFO,DataTransformer,Data processing started
2022-01-12 19:05:00,ERROR,DataTransformer,Error processing data
2022-01-12 19:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-12 19:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-13 20:00:00,INFO,DataTransformer,Data processing started
2022-01-13 20:05:00,ERROR,DataTransformer,Error processing data
2022-01-13 20:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-13 20:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-14 21:00:00,INFO,DataTransformer,Data processing started
2022-01-14 21:05:00,ERROR,DataTransformer,Error processing data
2022-01-14 21:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-14 21:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-15 22:00:00,INFO,DataTransformer,Data processing started
2022-01-15 22:05:00,ERROR,DataTransformer,Error processing data
2022-01-15 22:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-15 22:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-16 23:00:00,INFO,DataTransformer,Data processing started
2022-01-16 23:05:00,ERROR,DataTransformer,Error processing data
2022-01-16 23:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-16 23:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-17 00:00:00,INFO,DataTransformer,Data processing started
2022-01-17 00:05:00,ERROR,DataTransformer,Error processing data
2022-01-17 00:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-17 00:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-18 01:00:00,INFO,DataTransformer,Data processing started
2022-01-18 01:05:00,ERROR,DataTransformer,Error processing data
2022-01-18 01:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-18 01:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-19 02:00:00,INFO,DataTransformer,Data processing started
2022-01-19 02:05:00,ERROR,DataTransformer,Error processing data
2022-01-19 02:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-19 02:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-20 03:00:00,INFO,DataTransformer,Data processing started
2022-01-20 03:05:00,ERROR,DataTransformer,Error processing data
2022-01-20 03:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-20 03:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-21 04:00:00,INFO,DataTransformer,Data processing started
2022-01-21 04:05:00,ERROR,DataTransformer,Error processing data
2022-01-21 04:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-21 04:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-22 05:00:00,INFO,DataTransformer,Data processing started
2022-01-22 05:05:00,ERROR,DataTransformer,Error processing data
2022-01-22 05:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-22 05:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-23 06:00:00,INFO,DataTransformer,Data processing started
2022-01-23 06:05:00,ERROR,DataTransformer,Error processing data
2022-01-23 06:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-23 06:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-24 07:00:00,INFO,DataTransformer,Data processing started
2022-01-24 07:05:00,ERROR,DataTransformer,Error processing data
2022-01-24 07:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-24 07:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-25 08:00:00,INFO,DataTransformer,Data processing started
2022-01-25 08:05:00,ERROR,DataTransformer,Error processing data
2022-01-25 08:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-25 08:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-26 09:00:00,INFO,DataTransformer,Data processing started
2022-01-26 09:05:00,ERROR,DataTransformer,Error processing data
2022-01-26 09:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-26 09:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-27 10:00:00,INFO,DataTransformer,Data processing started
2022-01-27 10:05:00,ERROR,DataTransformer,Error processing data
2022-01-27 10:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-27 10:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-28 11:00:00,INFO,DataTransformer,Data processing started
2022-01-28 11:05:00,ERROR,DataTransformer,Error processing data
2022-01-28 11:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-28 11:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-29 12:00:00,INFO,DataTransformer,Data processing started
2022-01-29 12:05:00,ERROR,DataTransformer,Error processing data
2022-01-29 12:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-29 12:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-30 13:00:00,INFO,DataTransformer,Data processing started
2022-01-30 13:05:00,ERROR,DataTransformer,Error processing data
2022-01-30 13:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-30 13:15:00,WARNING,DataTransformer,Data quality check failed
2022-01-31 14:00:00,INFO,DataTransformer,Data processing started
2022-01-31 14:05:00,ERROR,DataTransformer,Error processing data
2022-01-31 14:10:00,INFO,DataTransformer,Data processing completed successfully
2022-01-31 14:15:00,WARNING,DataTransformer,Data quality check failed`;

const rows = rawText.split("\n").slice(1); // Split the text by lines and remove the header row

const logData = rows.map((row) => {
  const [timestamp, status, transformer, message] = row.split(",");
  return { timestamp, status, transformer, message };
});

export { logData };
