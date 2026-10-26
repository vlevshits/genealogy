import React, { useState } from 'react';
import { ChevronRight, FolderTree } from 'lucide-react';
import { cn } from '../../lib/utils';

export interface TreeNode {
  id: string;
  label: string;
  badge?: string | number;
  children?: TreeNode[];
}

export interface TreeViewProps {
  nodes: TreeNode[];
  selectedId?: string;
  onSelect?: (node: TreeNode) => void;
  ariaLabel?: string;
  className?: string;
}

const TreeBranch: React.FC<{
  node: TreeNode;
  level: number;
  selectedId?: string;
  onSelect?: (node: TreeNode) => void;
}> = ({ node, level, selectedId, onSelect }) => {
  const [expanded, setExpanded] = useState(level === 0);
  const hasChildren = Boolean(node.children && node.children.length > 0);
  const isSelected = selectedId === node.id;

  return (
    <li role="treeitem" aria-expanded={hasChildren ? expanded : undefined} aria-selected={isSelected}>
      <div
        style={{ paddingLeft: `${level * 16 + 8}px` }}
        onClick={() => {
          if (hasChildren) setExpanded((e) => !e);
          onSelect?.(node);
        }}
        onKeyDown={(e) => {
          if (e.key === 'ArrowRight' && hasChildren) setExpanded(true);
          if (e.key === 'ArrowLeft' && hasChildren) setExpanded(false);
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            onSelect?.(node);
          }
        }}
        tabIndex={0}
        className={cn(
          'flex items-center justify-between gap-2 rounded-lg py-1.5 pr-3 text-xs sm:text-sm cursor-pointer transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-focus',
          isSelected ? 'bg-brand/10 text-brand font-semibold' : 'text-primary hover:bg-surface-muted'
        )}
      >
        <span className="flex items-center gap-1.5 truncate">
          {hasChildren ? (
            <ChevronRight
              className={cn(
                'h-3.5 w-3.5 text-tertiary transition-transform',
                expanded && 'rotate-90'
              )}
            />
          ) : (
            <FolderTree className="h-3.5 w-3.5 text-tertiary" />
          )}
          <span className="truncate">{node.label}</span>
        </span>
        {node.badge !== undefined && (
          <span className="rounded bg-surface-muted px-1.5 py-0.5 font-mono text-[11px] text-secondary">
            {node.badge}
          </span>
        )}
      </div>
      {hasChildren && expanded && (
        <ul role="group" className="mt-0.5 space-y-0.5">
          {node.children!.map((child) => (
            <TreeBranch
              key={child.id}
              node={child}
              level={level + 1}
              selectedId={selectedId}
              onSelect={onSelect}
            />
          ))}
        </ul>
      )}
    </li>
  );
};

export const TreeView: React.FC<TreeViewProps> = ({
  nodes,
  selectedId,
  onSelect,
  ariaLabel = 'Hierarchy tree',
  className,
}) => {
  return (
    <ul
      role="tree"
      aria-label={ariaLabel}
      className={cn('space-y-0.5 rounded-xl border border-border bg-surface p-2', className)}
    >
      {nodes.map((node) => (
        <TreeBranch
          key={node.id}
          node={node}
          level={0}
          selectedId={selectedId}
          onSelect={onSelect}
        />
      ))}
    </ul>
  );
};
