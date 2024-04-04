import { Handle, HandleProps } from "reactflow";

const CustomHandle: React.FC<HandleProps> = props => {
  return <Handle {...props} className="bg-zinc-100 border-zinc-100" />;
};

export default CustomHandle;
