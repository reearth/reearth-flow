import type { FieldNode } from "@flow/lib/schemaForm";

import { ArrayField } from "./ArrayField";
import { BooleanField } from "./BooleanField";
import { ColorField } from "./ColorField";
import { EnumField } from "./EnumField";
import { ExprField } from "./ExprField";
import { MapField } from "./MapField";
import { NumberField } from "./NumberField";
import { ObjectField } from "./ObjectField";
import { StringField } from "./StringField";
import type { FieldProps } from "./types";
import { UnionField } from "./UnionField";
import { UnsupportedField } from "./UnsupportedField";
import { WysiwygField } from "./WysiwygField";

/**
 * One switch over the compiled field kinds.
 *
 * This is the whole dispatch mechanism — no widget registry, no template
 * lookup, no `ui:` protocol. What a field is was decided by `compile`, so the
 * only question left here is which component draws it.
 */
const Field: React.FC<FieldProps> = (props) => {
  const node: FieldNode = props.node;

  switch (node.kind) {
    case "string":
      return <StringField {...props} node={node} />;
    case "number":
      return <NumberField {...props} node={node} />;
    case "boolean":
      return <BooleanField {...props} node={node} />;
    case "enum":
      return <EnumField {...props} node={node} />;
    case "expr":
      return <ExprField {...props} node={node} />;
    case "color":
      return <ColorField {...props} node={node} />;
    case "wysiwyg":
      return <WysiwygField {...props} node={node} />;
    case "array":
      return <ArrayField {...props} node={node} />;
    case "map":
      return <MapField {...props} node={node} />;
    case "object":
      return <ObjectField {...props} node={node} />;
    case "union":
      return <UnionField {...props} node={node} />;
    case "unsupported":
      return <UnsupportedField {...props} node={node} />;
  }
};

export { Field };
