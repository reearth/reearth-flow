const FieldWrapper: React.FC<{ children?: React.ReactNode }> = ({ children }) => {
  return <div className="flex w-full justify-between items-center">{children}</div>;
};

export { FieldWrapper };
