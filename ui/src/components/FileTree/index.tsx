import { CaretRight, Icon } from "@phosphor-icons/react";
import * as AccordionPrimitive from "@radix-ui/react-accordion";
import { XYPosition } from "@xyflow/react";
import { forwardRef, useCallback, useMemo, useState } from "react";
import useResizeObserver from "use-resize-observer";

import { cn } from "@flow/lib/utils";

type TreeDataItem = {
  id: string;
  name: string;
  icon?: Icon;
  type?: string;
  position?: XYPosition;
  measured?: { width: number; height: number };
  children?: TreeDataItem[];
};

type TreeProps = React.HTMLAttributes<HTMLDivElement> & {
  data: TreeDataItem[] | TreeDataItem;
  initialSelectedItemId?: string;
  onSelectChange?: (item: TreeDataItem | undefined) => void;
  expandAll?: boolean;
  folderIcon?: Icon;
  itemIcon?: Icon;
};

const Tree = forwardRef<HTMLDivElement, TreeProps>(
  (
    {
      data,
      initialSelectedItemId,
      onSelectChange,
      expandAll,
      folderIcon,
      itemIcon,
      className,
      ...props
    },
    ref,
  ) => {
    const [selectedItemId, setSelectedItemId] = useState<string | undefined>(
      initialSelectedItemId,
    );

    const handleSelectChange = useCallback(
      (item: TreeDataItem | undefined) => {
        setSelectedItemId(item?.id);
        if (onSelectChange) {
          onSelectChange(item);
        }
      },
      [onSelectChange],
    );

    const expandedItemIds = useMemo(() => {
      if (!initialSelectedItemId) {
        return [] as string[];
      }

      const ids: string[] = [];

      function walkTreeItems(
        items: TreeDataItem[] | TreeDataItem,
        targetId: string,
      ) {
        if (items instanceof Array) {
          // eslint-disable-next-line @typescript-eslint/prefer-for-of
          for (let i = 0; i < items.length; i++) {
            ids.push(items[i].id);
            if (walkTreeItems(items[i], targetId) && !expandAll) {
              return true;
            }
            if (!expandAll) ids.pop();
          }
        } else if (!expandAll && items.id === targetId) {
          return true;
        } else if (items.children) {
          return walkTreeItems(items.children, targetId);
        }
      }

      walkTreeItems(data, initialSelectedItemId);
      return ids;
    }, [data, expandAll, initialSelectedItemId]);

    const { ref: refRoot } = useResizeObserver();
    // const { ref: refRoot, width, height } = useResizeObserver();

    return (
      <div ref={refRoot} className={cn("overflow-auto", className)}>
        <div className="relative">
          <TreeItem
            data={data}
            ref={ref}
            selectedItemId={selectedItemId}
            handleSelectChange={handleSelectChange}
            expandedItemIds={expandedItemIds}
            FolderIcon={folderIcon}
            ItemIcon={itemIcon}
            {...props}
          />
        </div>
      </div>
    );
  },
);

Tree.displayName = "Tree";

type TreeItemProps = TreeProps & {
  selectedItemId?: string;
  handleSelectChange: (item: TreeDataItem | undefined) => void;
  expandedItemIds: string[];
  FolderIcon?: Icon;
  ItemIcon?: Icon;
};

const TreeItem = forwardRef<HTMLDivElement, TreeItemProps>(
  (
    {
      className,
      data,
      selectedItemId,
      handleSelectChange,
      expandedItemIds,
      FolderIcon,
      ItemIcon,
      ...props
    },
    ref,
  ) => {
    return (
      <div ref={ref} role="tree" className={className} {...props}>
        <ul>
          {data instanceof Array ? (
            data.map((item) => (
              <li key={item.id}>
                {item.children ? (
                  <AccordionPrimitive.Root
                    type="multiple"
                    defaultValue={expandedItemIds}>
                    <AccordionPrimitive.Item value={item.id}>
                      <AccordionTrigger
                        className={cn(
                          "px-2 before:absolute before:left-0 before:-z-10 before:h-[1.75rem] before:w-full before:rounded-md before:bg-primary before:opacity-0 hover:before:opacity-100",
                          selectedItemId === item.id &&
                            "before:rounded-md before:border before:border-l-2 before:border-accent before:border-l-logo/30 before:bg-primary before:opacity-100",
                        )}
                        onClick={() => handleSelectChange(item)}>
                        {item.icon && (
                          <item.icon
                            className={cn("mr-2 h-4 w-4 shrink-0")}
                            weight="thin"
                            aria-hidden="true"
                          />
                        )}
                        {!item.icon && FolderIcon && (
                          <FolderIcon
                            className="mr-2 size-4 shrink-0 "
                            weight="thin"
                            aria-hidden="true"
                          />
                        )}
                        <span className="truncate text-xs dark:font-extralight">
                          {item.name}
                        </span>
                      </AccordionTrigger>
                      <AccordionContent className="ml-4 border-l pl-6">
                        <TreeItem
                          className="-ml-4"
                          data={item.children ? item.children : item}
                          selectedItemId={selectedItemId}
                          handleSelectChange={handleSelectChange}
                          expandedItemIds={expandedItemIds}
                          FolderIcon={FolderIcon}
                          ItemIcon={ItemIcon}
                        />
                      </AccordionContent>
                    </AccordionPrimitive.Item>
                  </AccordionPrimitive.Root>
                ) : (
                  <Leaf
                    item={item}
                    isSelected={selectedItemId === item.id}
                    onClick={() => handleSelectChange(item)}
                    Icon={ItemIcon}
                  />
                )}
              </li>
            ))
          ) : (
            <li>
              <Leaf
                item={data}
                isSelected={selectedItemId === data.id}
                onClick={() => handleSelectChange(data)}
                Icon={ItemIcon}
              />
            </li>
          )}
        </ul>
      </div>
    );
  },
);

TreeItem.displayName = "TreeItem";

const Leaf = forwardRef<
  HTMLDivElement,
  React.HTMLAttributes<HTMLDivElement> & {
    item: TreeDataItem;
    isSelected?: boolean;
    Icon?: Icon;
  }
>(({ className, item, isSelected, Icon, ...props }, ref) => {
  return (
    <div
      ref={ref}
      className={cn(
        "flex cursor-pointer items-center px-2 py-2",
        "before:absolute before:right-1 before:left-0 before:-z-10 before:h-[1.75rem] before:w-full before:rounded-md before:bg-primary before:opacity-0 hover:before:opacity-100",
        className,
        isSelected &&
          "before:rounded-md before:border before:border-l-2 before:border-accent before:border-l-logo/30 before:bg-primary before:opacity-100",
      )}
      {...props}>
      {item.icon && (
        <item.icon
          className={cn("mr-2 h-4 w-4 shrink-0")}
          weight="thin"
          aria-hidden="true"
        />
      )}
      {!item.icon && Icon && (
        <Icon
          className="mr-2 size-4 shrink-0 "
          weight="thin"
          aria-hidden="true"
        />
      )}
      <span className="grow truncate text-xs dark:font-extralight">
        {item.name}
      </span>
    </div>
  );
});

Leaf.displayName = "Leaf";

const AccordionTrigger = forwardRef<
  React.ElementRef<typeof AccordionPrimitive.Trigger>,
  React.ComponentPropsWithoutRef<typeof AccordionPrimitive.Trigger>
>(({ className, children, ...props }, ref) => (
  <AccordionPrimitive.Header>
    <AccordionPrimitive.Trigger
      ref={ref}
      className={cn(
        "flex w-full flex-1 items-center py-2 transition-all [&[data-state=open]>svg]:last:rotate-90",
        className,
      )}
      {...props}>
      {children}
      <CaretRight className="ml-auto size-4 shrink-0 transition-transform duration-200" />
    </AccordionPrimitive.Trigger>
  </AccordionPrimitive.Header>
));
AccordionTrigger.displayName = AccordionPrimitive.Trigger.displayName;

const AccordionContent = forwardRef<
  React.ElementRef<typeof AccordionPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof AccordionPrimitive.Content>
>(({ className, children, ...props }, ref) => (
  <AccordionPrimitive.Content
    ref={ref}
    className={cn(
      "overflow-hidden text-sm transition-all data-[state=closed]:animate-accordion-up data-[state=open]:animate-accordion-down",
      className,
    )}
    {...props}>
    <div className="pt-0 pb-1">{children}</div>
  </AccordionPrimitive.Content>
));
AccordionContent.displayName = AccordionPrimitive.Content.displayName;

export { Tree, type TreeDataItem };
