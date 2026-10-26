import React, { useState } from 'react';
import { ImageOff } from 'lucide-react';
import { cn } from '../../lib/utils';

export interface ImageProps extends React.ImgHTMLAttributes<HTMLImageElement> {
  aspectRatio?: '1:1' | '4:3' | '16:9' | '3:4';
  fallbackLabel?: string;
}

export const Image: React.FC<ImageProps> = ({
  src,
  alt = '',
  aspectRatio = '4:3',
  fallbackLabel,
  className,
  ...props
}) => {
  const [loaded, setLoaded] = useState(false);
  const [errored, setErrored] = useState(false);

  const ratioClasses = {
    '1:1': 'aspect-square',
    '4:3': 'aspect-4/3',
    '16:9': 'aspect-video',
    '3:4': 'aspect-3/4',
  };

  return (
    <div
      className={cn(
        'relative overflow-hidden rounded-2xl border border-border bg-surface-muted',
        ratioClasses[aspectRatio],
        className
      )}
    >
      {!errored && src ? (
        <img
          src={src}
          alt={alt}
          loading="lazy"
          onLoad={() => setLoaded(true)}
          onError={() => setErrored(true)}
          className={cn(
            'h-full w-full object-cover transition-all duration-300',
            loaded ? 'blur-0 scale-100' : 'blur-xs scale-105'
          )}
          {...props}
        />
      ) : (
        <div className="flex h-full w-full flex-col items-center justify-center gap-1.5 p-4 text-center text-xs text-tertiary">
          <ImageOff className="h-6 w-6" />
          {fallbackLabel && <span>{fallbackLabel}</span>}
        </div>
      )}
    </div>
  );
};
