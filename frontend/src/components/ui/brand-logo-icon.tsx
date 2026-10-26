import React from 'react';
import { cn } from '../../lib/utils';

export interface BrandLogoIconProps extends React.SVGProps<SVGSVGElement> {
  size?: number | string;
}

/**
 * Reusable organic Family & Heritage Tree brand icon.
 * Renders a deciduous family tree canopy with trunk, roots, and ancestral crown nodes
 * so it unmistakably represents a family tree rather than a Git branch.
 */
export const BrandLogoIcon = React.forwardRef<SVGSVGElement, BrandLogoIconProps>(
  ({ className, size, ...props }, ref) => (
    <svg
      ref={ref}
      xmlns="http://www.w3.org/2000/svg"
      width={size ?? 20}
      height={size ?? 20}
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.9}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className={cn('shrink-0', className)}
      {...props}
    >
      {/* Organic three-lobed deciduous family tree crown */}
      <path d="M12 3a4.5 4.5 0 0 0-4.1 2.65A4 4 0 0 0 5 9.5c0 2.05 1.54 3.74 3.53 3.97h6.94A4 4 0 0 0 19 9.5a4 4 0 0 0-2.9-3.85A4.5 4.5 0 0 0 12 3Z" />
      {/* Trunk */}
      <path d="M12 11v9" />
      {/* Branching boughs inside the crown */}
      <path d="M12 14.5 9.2 11.8" />
      <path d="M12 13.5 14.8 11" />
      {/* Grounded heritage roots */}
      <path d="M8.5 21c1.1-.8 2.2-1 3.5-1s2.4.2 3.5 1" />
      {/* Ancestral generation nodes in the canopy */}
      <circle cx="12" cy="7.2" r="0.9" fill="currentColor" stroke="none" />
      <circle cx="9.2" cy="9.8" r="0.85" fill="currentColor" stroke="none" />
      <circle cx="14.8" cy="9.4" r="0.85" fill="currentColor" stroke="none" />
    </svg>
  )
);

BrandLogoIcon.displayName = 'BrandLogoIcon';
