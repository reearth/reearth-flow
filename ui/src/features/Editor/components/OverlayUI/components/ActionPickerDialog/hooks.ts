import { EdgeChange, useReactFlow, XYPosition } from "@xyflow/react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useDoubleClick } from "@flow/hooks";
import { useAction } from "@flow/lib/fetch";
import { useT } from "@flow/lib/i18n";
import i18n from "@flow/lib/i18n/i18n";
import { buildNewCanvasNode } from "@flow/lib/reactFlow";
import { ActionNodeType, Edge, Node } from "@flow/types";
import { generateUUID } from "@flow/utils";
import { getRandomNumberInRange } from "@flow/utils/getRandomNumberInRange";

type CategoryFiltering = string;
type TagFiltering = string;
export default ({
  openedActionType,
  isMainWorkflow,
  nodes,
  selectedNodeIds,
  edges,
  openNodePickerViaShortcut,
  onNodesAdd,
  onEdgesAdd,
  onEdgesChange,
  onClose,
}: {
  openedActionType: {
    position: XYPosition;
    nodeType: ActionNodeType;
  };
  nodes: Node[];
  selectedNodeIds: string[];
  edges?: Edge[];
  isMainWorkflow: boolean;
  openNodePickerViaShortcut: boolean;
  onNodesAdd: (nodes: Node[]) => void;
  onEdgesAdd?: (edges: Edge[]) => void;
  onEdgesChange?: (changes: EdgeChange[]) => void;
  onClose: () => void;
}) => {
  const t = useT();
  const [searchTerm, setSearchTerm] = useState("");
  const [currentActionByTypes, setCurrentActionByTypes] = useState<
    ActionNodeType[]
  >([openedActionType.nodeType]);
  const [currentCategories, setCurrentCategories] = useState<
    CategoryFiltering[]
  >([]);
  const [currentTags, setCurrentTags] = useState<TagFiltering[]>([]);

  const actionTypes: { value: ActionNodeType; label: string }[] = [
    { value: "reader", label: t("Reader") },
    { value: "transformer", label: t("Transformer") },
    { value: "writer", label: t("Writer") },
  ];

  const actionCategories: { value: CategoryFiltering; label: string }[] = [
    { value: "3D", label: t("3D") },
    { value: "Attribute", label: t("Attribute") },
    { value: "Debug", label: t("Debug") },
    { value: "Feature", label: t("Feature") },
    { value: "File", label: t("File") },
    { value: "Filter", label: t("Filter") },
    { value: "Geometry", label: t("Geometry") },
    { value: "Input", label: t("Input") },
    { value: "Merge", label: t("Merge") },
    { value: "Output", label: t("Output") },
    { value: "PLATEAU", label: t("PLATEAU") },
    { value: "Python", label: t("Python") },
    { value: "Script", label: t("Script") },
    { value: "Transform", label: t("Transform") },
    { value: "Wasm", label: t("Wasm") },
    { value: "Web", label: t("Web") },
  ];

  const actionTags: { value: TagFiltering; label: string }[] = [
    { value: "2d", label: t("2D") },
    { value: "3d", label: t("3D") },
    { value: "3d-tiles", label: t("3D Tiles") },
    { value: "aggregate", label: t("Aggregate") },
    { value: "area", label: t("Area") },
    { value: "citygml", label: t("CityGML") },
    { value: "compression", label: t("Compression") },
    { value: "csv", label: t("CSV") },
    { value: "database", label: t("Database") },
    { value: "decompose", label: t("Decompose") },
    { value: "file", label: t("File") },
    { value: "file-system", label: t("File System") },
    { value: "geojson", label: t("GeoJSON") },
    { value: "geometry", label: t("Geometry") },
    { value: "geopackage", label: t("GeoPackage") },
    { value: "hierarchy", label: t("Hierarchy") },
    { value: "image", label: t("Image") },
    { value: "intersection", label: t("Intersection") },
    { value: "join", label: t("Join") },
    { value: "json", label: t("JSON") },
    { value: "list", label: t("List") },
    { value: "lod", label: t("LOD") },
    { value: "mapping", label: t("Mapping") },
    { value: "measurement", label: t("Measurement") },
    { value: "mvt", label: t("MVT") },
    { value: "normal", label: t("Normal") },
    { value: "path", label: t("Path") },
    { value: "projection", label: t("Projection") },
    { value: "raster", label: t("Raster") },
    { value: "ray", label: t("Ray") },
    { value: "routing", label: t("Routing") },
    { value: "scripting", label: t("Scripting") },
    { value: "shapefile", label: t("Shapefile") },
    { value: "sort", label: t("Sort") },
    { value: "split", label: t("Split") },
    { value: "statistics", label: t("Statistics") },
    { value: "texture", label: t("Texture") },
    { value: "validate", label: t("Validate") },
    { value: "xml", label: t("XML") },
  ];

  const containerRef = useRef<HTMLDivElement>(null);
  const itemRefs = useRef<(HTMLDivElement | null)[]>([]);
  // const { handleNodeDropInBatch } = useBatch();
  const { screenToFlowPosition } = useReactFlow();
  const { useGetActionsSegregated } = useAction(i18n.language);
  const filterConfig = useMemo(
    () => ({
      isMainWorkflow,
      searchTerm,
      types: currentActionByTypes.length ? currentActionByTypes : undefined,
      categories: currentCategories.length ? currentCategories : undefined,
      tags: currentTags.length ? currentTags : undefined,
    }),
    [
      isMainWorkflow,
      searchTerm,
      currentActionByTypes,
      currentCategories,
      currentTags,
    ],
  );
  const { actions: segregatedActions } = useGetActionsSegregated(filterConfig);

  const actionsList = useMemo(() => {
    if (currentActionByTypes.length) {
      return currentActionByTypes.flatMap(
        (type) => segregatedActions?.byType[type] ?? [],
      );
    }
    return Object.values(segregatedActions?.byType ?? {}).flatMap(
      (a) => a ?? [],
    );
  }, [currentActionByTypes, segregatedActions]);

  const [selectedIndex, setSelectedIndex] = useState(-1);
  const [selected, setSelected] = useState<string | undefined>();

  const handleSearchTerm = (newSearchTerm: string) => {
    setSearchTerm(newSearchTerm);
    setSelectedIndex(-1);
  };

  useEffect(() => {
    const selectedItem = itemRefs.current[selectedIndex];
    if (selectedItem && containerRef.current) {
      requestAnimationFrame(() => {
        selectedItem.scrollIntoView({
          behavior: "smooth",
          block: "center",
          inline: "nearest",
        });
      });
    }
  }, [selectedIndex]);

  const [handleSingleClick, handleDoubleClick] = useDoubleClick(
    (name?: string) => {
      if (!name) return;
      const idx = actionsList.findIndex((a) => a.name === name);
      setSelectedIndex((prev) => (prev === idx ? -1 : idx));
    },
    async (name?: string) => {
      if (!name) return;
      // If the position is 0,0 then place it in the center of the screen as this is using shortcut creation and not dnd
      const randomX = getRandomNumberInRange(50, 200);
      const randomY = getRandomNumberInRange(50, 200);
      const lastSelectedNode = nodes.find(
        (n) => n.id === selectedNodeIds[selectedNodeIds.length - 1],
      );
      const outgoingEdges = lastSelectedNode
        ? edges?.filter((e) => e.source === lastSelectedNode.id)
        : undefined;

      let position: XYPosition;
      if (lastSelectedNode && openNodePickerViaShortcut) {
        position = outgoingEdges?.length
          ? {
              x: lastSelectedNode.position.x + 125,
              y: lastSelectedNode.position.y + 75,
            }
          : {
              x: lastSelectedNode.position.x + 250,
              y: lastSelectedNode.position.y,
            };
      } else if (
        openedActionType.position.x === 0 &&
        openedActionType.position.y === 0
      ) {
        position = screenToFlowPosition({
          x: window.innerWidth / 2 + randomX,
          y: window.innerHeight / 2 - randomY,
        });
      } else {
        position = openedActionType.position;
      }

      const newNode = await buildNewCanvasNode({ position, type: name });
      if (!newNode) return;

      onNodesAdd([newNode]);

      if (lastSelectedNode && openNodePickerViaShortcut) {
        if (lastSelectedNode.type !== "writer" && newNode.type !== "reader") {
          onEdgesAdd?.([
            {
              id: generateUUID(),
              source: lastSelectedNode.id,
              target: newNode.id,
            },
          ]);
        }

        if (
          outgoingEdges?.length &&
          lastSelectedNode.type !== "writer" &&
          newNode.type !== "writer" &&
          newNode.type !== "reader"
        ) {
          const removeChanges: EdgeChange[] = outgoingEdges.map((e) => ({
            id: e.id,
            type: "remove" as const,
          }));
          const addChanges: EdgeChange[] = outgoingEdges.map((e) => ({
            type: "add" as const,
            item: {
              id: generateUUID(),
              source: newNode.id,
              target: e.target,
              sourceHandle: e.sourceHandle ?? null,
              targetHandle: e.targetHandle ?? null,
            },
          }));
          onEdgesChange?.([...removeChanges, ...addChanges]);
        }
      }

      // TODO - add drop in batch support
      // onNodesChange(handleNodeDropInBatch(newNode, newNodes));
      onClose();

      const focusNewNode = () => {
        const el = document.querySelector<HTMLElement>(
          `[data-id="${newNode.id}"]`,
        );
        if (el) {
          el.focus();
        } else {
          requestAnimationFrame(focusNewNode);
        }
      };
      requestAnimationFrame(focusNewNode);
    },
  );

  useEffect(() => {
    setSelected(actionsList?.[selectedIndex]?.name || undefined);
  }, [selectedIndex, actionsList]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const target = e.target as HTMLElement;
      if (
        target.closest("[data-filter-area]") ||
        target.closest('[role="combobox"]') ||
        target.closest('[role="listbox"]') ||
        target.closest('[role="option"]')
      ) {
        return;
      }

      const currentActionsList = actionsList || [];

      switch (e.key) {
        case "Enter":
          e.preventDefault();
          handleDoubleClick(selected);
          break;
        case "ArrowUp":
          e.preventDefault();
          setSelectedIndex(
            selectedIndex === 0 ? selectedIndex : selectedIndex - 1,
          );
          break;
        case "ArrowDown":
          e.preventDefault();
          setSelectedIndex(
            selectedIndex === (currentActionsList?.length || 1) - 1
              ? selectedIndex
              : selectedIndex + 1,
          );
          break;
      }
    };

    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [actionsList, selectedIndex, selected, handleDoubleClick]);

  const handleActionTypeToggle = useCallback((type: string) => {
    const nodeType = type as ActionNodeType;
    setCurrentActionByTypes((prev) =>
      prev.includes(nodeType)
        ? prev.filter((t) => t !== nodeType)
        : [...prev, nodeType],
    );
    setSelectedIndex(-1);
    containerRef.current?.scrollTo({ top: 0, behavior: "smooth" });
  }, []);

  const handleCategoryToggle = useCallback((category: CategoryFiltering) => {
    setCurrentCategories((prev) =>
      prev.includes(category)
        ? prev.filter((c) => c !== category)
        : [...prev, category],
    );
    setSelectedIndex(-1);
    containerRef.current?.scrollTo({ top: 0, behavior: "smooth" });
  }, []);

  const handleTagToggle = useCallback((tag: TagFiltering) => {
    setCurrentTags((prev) =>
      prev.includes(tag) ? prev.filter((t) => t !== tag) : [...prev, tag],
    );
    setSelectedIndex(-1);
    containerRef.current?.scrollTo({ top: 0, behavior: "smooth" });
  }, []);

  const handleClearFilters = useCallback(() => {
    setCurrentActionByTypes([]);
    setCurrentCategories([]);
    setCurrentTags([]);
    setSelectedIndex(-1);
    containerRef.current?.scrollTo({ top: 0, behavior: "smooth" });
  }, []);

  return {
    actionsList,
    containerRef,
    itemRefs,
    selected,
    currentActionByTypes,
    currentCategories,
    currentTags,
    actionTypes,
    actionCategories,
    actionTags,
    handleSearchTerm,
    handleSingleClick,
    handleDoubleClick,
    handleActionTypeToggle,
    handleCategoryToggle,
    handleTagToggle,
    handleClearFilters,
  };
};
