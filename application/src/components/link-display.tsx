import { useState } from "react";
import { ExternalLink } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface LinkDisplayProps {
  url: string;
}

export function LinkDisplay({ url }: LinkDisplayProps) {
  const [isTooltipOpen, setIsTooltipOpen] = useState(false);

  const getDomainName = (url: string) => {
    try {
      const domain = new URL(url).hostname;
      return domain.replace("www.", "");
    } catch {
      return "Unknown";
    }
  };

  const domainName = getDomainName(url);

  return (
    <TooltipProvider>
      <Tooltip open={isTooltipOpen} onOpenChange={setIsTooltipOpen}>
        <TooltipTrigger asChild>
          <Button
            variant="outline"
            size="sm"
            className="max-w-[200px]"
            onClick={(e) => {
              e.stopPropagation();
              openUrl(url);
            }}
          >
            <ExternalLink className="w-4 h-4 mr-2 shrink-0" />
            <span className="truncate">{domainName}</span>
          </Button>
        </TooltipTrigger>
        <TooltipContent side="bottom" align="start">
          <p className="max-w-xs break-all">{url}</p>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  );
}
