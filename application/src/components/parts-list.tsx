import { ScrollArea } from "@/components/ui/scroll-area";
import { Card, CardContent } from "@/components/ui/card";
import { Loader2, X, ExternalLink } from "lucide-react";
import { Button } from "@/components/ui/button";
import useGetExhibitParts from "@/hooks/data/queries/exhibits/useGetExhibitParts";
import useEditPart from "@/hooks/data/mutations/parts/useEditPart";
import { useState } from "react";
import { LinkDisplay } from "@/components/link-display";

interface PartsListProps {
  refetchPartIds: () => void;
  partIds: string[];
  exhibitId: string;
}

export function PartsList({
  partIds,
  exhibitId,
  refetchPartIds,
}: PartsListProps) {
  const {
    data: parts,
    isLoading,
    isError,
    refetch,
  } = useGetExhibitParts(partIds);
  const editPartMutation = useEditPart();
  const [removingPartId, setRemovingPartId] = useState<string | null>(null);

  const handleRemovePart = async (partId: string) => {
    setRemovingPartId(partId);
    try {
      const partToUpdate = parts?.find((part) => part.id === partId);
      if (partToUpdate) {
        await editPartMutation.mutateAsync({
          id: partId,
          payload: {
            name: partToUpdate.name,
            link: partToUpdate.link,
            exhibitIds: partToUpdate.exhibit_ids
              .filter((id) => id !== exhibitId)
              .map((id) => Number(id)),
          },
        });
        refetch();
        refetchPartIds();
      }
    } catch (error) {
      console.error("Failed to remove part from exhibit:", error);
    } finally {
      setRemovingPartId(null);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-48">
        <Loader2 className="h-8 w-8 animate-spin text-primary" />
      </div>
    );
  }

  if (isError) {
    return (
      <Card className="h-48">
        <CardContent className="h-full flex items-center justify-center">
          <p className="text-sm text-destructive">
            Error loading parts. Please try again.
          </p>
        </CardContent>
      </Card>
    );
  }

  if (!parts || parts.length === 0) {
    return (
      <Card className="h-48">
        <CardContent className="h-full flex items-center justify-center">
          <p className="text-sm text-muted-foreground">
            No parts connected to this exhibit.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card className="h-48">
      <ScrollArea className="h-full w-full px-3 py-2">
        <div className="space-y-2">
          {parts.map((part, index) => (
            <div key={part.id}>
              {index > 0 && <div className="h-px bg-border my-2" />}
              <Card className="border-0 shadow-xs">
                <CardContent className="p-3 flex justify-between items-center">
                  <div className="grow mr-2">
                    <h5 className="font-medium text-sm truncate">
                      {part.name}
                    </h5>
                  </div>
                  <div className="flex items-center space-x-2 shrink-0">
                    <LinkDisplay url={part.link} />
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => handleRemovePart(part.id)}
                      disabled={removingPartId === part.id}
                      className="h-8 w-8"
                    >
                      {removingPartId === part.id ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        <X className="h-4 w-4" />
                      )}
                      <span className="sr-only">Remove</span>
                    </Button>
                  </div>
                </CardContent>
              </Card>
            </div>
          ))}
        </div>
      </ScrollArea>
    </Card>
  );
}
