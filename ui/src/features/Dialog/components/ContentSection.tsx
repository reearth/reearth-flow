type Props = {
  className?: string;
  title: string;
  content: React.ReactNode;
};

const ContentSection: React.FC<Props> = ({ className, title, content }) => (
  <div className="mb-4 px-6 flex-1">
    <h2 className="text-xs uppercase">{title}</h2>
    <div className={`border-t border-zinc-700 pl-4 pt-4 mt-2 ${className}`}>{content}</div>
  </div>
);

export { ContentSection };
