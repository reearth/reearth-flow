import { useCallback, useRef, useEffect } from "react";

import { TextArea } from "@flow/components";

import RhaiSyntaxHighlighter from "./RhaiSyntaxHighlighter";

type Props = {
  value: string;
  onChange: (value: string) => void;
  placeholder?: string;
  className?: string;
  "data-testid"?: string;
  "aria-label"?: string;
  "data-placeholder"?: string;
};

const RhaiCodeEditor: React.FC<Props> = ({
  value,
  onChange,
  placeholder,
  className = "",
  ...props
}) => {
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const highlightRef = useRef<HTMLDivElement>(null);
  const placeholderRef = useRef<HTMLDivElement>(null);

  // Sync scroll position between textarea and highlight overlay
  const handleScroll = useCallback(() => {
    if (textareaRef.current && highlightRef.current) {
      highlightRef.current.scrollTop = textareaRef.current.scrollTop;
      highlightRef.current.scrollLeft = textareaRef.current.scrollLeft;
    }
  }, []);

  // Sync positioning and styles exactly
  useEffect(() => {
    const syncStyles = () => {
      if (textareaRef.current && (highlightRef.current || placeholderRef.current)) {
        const textarea = textareaRef.current;
        const computedStyle = window.getComputedStyle(textarea);
        
        // Copy ALL relevant styles to ensure perfect alignment
        const stylesToCopy = [
          'fontSize', 'fontFamily', 'fontWeight', 'lineHeight', 'letterSpacing',
          'wordSpacing', 'tabSize', 'textIndent', 'textTransform',
          'padding', 'paddingTop', 'paddingRight', 'paddingBottom', 'paddingLeft',
          'border', 'borderWidth', 'borderStyle', 'borderColor',
          'borderTop', 'borderRight', 'borderBottom', 'borderLeft',
          'margin', 'marginTop', 'marginRight', 'marginBottom', 'marginLeft',
          'boxSizing', 'width', 'height'
        ];
        
        // Sync highlight overlay
        if (highlightRef.current) {
          const highlight = highlightRef.current;
          stylesToCopy.forEach(prop => {
            (highlight.style as any)[prop] = computedStyle.getPropertyValue(prop.replace(/([A-Z])/g, '-$1').toLowerCase());
          });
          
          highlight.style.position = 'absolute';
          highlight.style.top = '0';
          highlight.style.left = '0';
          highlight.style.pointerEvents = 'none';
          highlight.style.overflow = 'hidden';
          highlight.style.whiteSpace = 'pre-wrap';
          highlight.style.overflowWrap = 'break-word';
        }
        
        // Sync placeholder
        if (placeholderRef.current) {
          const placeholder = placeholderRef.current;
          stylesToCopy.forEach(prop => {
            (placeholder.style as any)[prop] = computedStyle.getPropertyValue(prop.replace(/([A-Z])/g, '-$1').toLowerCase());
          });
          
          placeholder.style.position = 'absolute';
          placeholder.style.top = '0px';
          placeholder.style.left = '0px';
          placeholder.style.pointerEvents = 'none';
          placeholder.style.overflow = 'hidden';
          placeholder.style.whiteSpace = 'pre-wrap';
          placeholder.style.overflowWrap = 'break-word';
          placeholder.style.color = 'rgb(107 114 128)'; // text-muted-foreground
        }
      }
    };
    
    // Sync immediately and on resize
    syncStyles();
    
    const resizeObserver = new ResizeObserver(syncStyles);
    if (textareaRef.current) {
      resizeObserver.observe(textareaRef.current);
    }
    
    return () => resizeObserver.disconnect();
  }, [value]);

  return (
    <div className={`relative ${className} flex`}>
      {/* Invisible textarea for input */}
      <TextArea
        ref={textareaRef}
        className="relative flex-1 resize-none border-transparent text-transparent caret-gray-900 selection:bg-blue-200 focus-visible:ring-0 dark:caret-gray-100 dark:selection:bg-blue-800"
        style={{ zIndex: 2 }}
        placeholder="" // Remove duplicate placeholder
        value={value}
        onChange={(e) => onChange(e.target.value)}
        onScroll={handleScroll}
        spellCheck={false}
        {...props}
      />

      {/* Syntax highlighted background - positioned exactly over textarea */}
      <div
        ref={highlightRef}
        className="pointer-events-none absolute bg-transparent"
        style={{
          zIndex: 1,
        }}>
        <RhaiSyntaxHighlighter 
          code={value} 
          className=""
        />
      </div>
      
      {/* Single placeholder when empty */}
      {!value && placeholder && (
        <div 
          ref={placeholderRef}
          className="pointer-events-none absolute text-muted-foreground"
          style={{ 
            zIndex: 0,
            top: 0,
            left: 0,
          }}
        >
          {placeholder}
        </div>
      )}
    </div>
  );
};

export default RhaiCodeEditor;