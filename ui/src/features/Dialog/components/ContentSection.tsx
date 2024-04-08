type Props = {
  title: string;
  content: React.ReactNode;
};

const ContentSection: React.FC<Props> = ({ title, content }) => (
  <div className="mb-4">
    <h2 className="text-xs uppercase">{title}</h2>
    <div className="border-t border-zinc-700 pl-4 py-4 mt-2">{content}</div>
  </div>
);

export { ContentSection };
