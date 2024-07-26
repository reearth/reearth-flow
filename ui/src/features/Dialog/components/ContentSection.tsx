type Props = {
  className?: string;
  title: string;
  content: React.ReactNode;
};

const ContentSection: React.FC<Props> = ({ className, title, content }) => (
  <div className="mb-4 flex-1 px-6">
    <h2 className="text-xs uppercase">{title}</h2>
    <div className={`mt-2 border-t pt-4 ${className}`}>{content}</div>
  </div>
);

export { ContentSection };
