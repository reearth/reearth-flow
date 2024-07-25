const FieldWrapper: React.FC<{ children?: React.ReactNode }> = ({ children }) => {
  return <div className="flex w-full items-center justify-between">{children}</div>;
};

export { FieldWrapper };
