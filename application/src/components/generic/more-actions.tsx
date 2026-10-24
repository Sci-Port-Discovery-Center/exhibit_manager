import { useState } from "react";
import { MoreVertical, Pencil, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import {
  Dialog,
  DialogTrigger,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
  DialogClose,
} from "@/components/ui/dialog";
import useDeletePart from "@/hooks/data/mutations/parts/useDeletePart";
import useDeleteExhibit from "@/hooks/data/mutations/exhibits/useDeleteExhibit";
import { GenericEditDialog } from "@/components/generic/edit-dialogue";

interface MoreActionsProps {
  id: string;
  type: "exhibit" | "part";
}

export function MoreActions({ id, type }: MoreActionsProps) {
  const deletePartMutation = useDeletePart();
  const deleteExhibitMutation = useDeleteExhibit();
  const [isEditDialogOpen, setIsEditDialogOpen] = useState(false);
  const [isDeleteDialogOpen, setIsDeleteDialogOpen] = useState(false);

  const handleEdit = (event: Event) => {
    event.preventDefault();
    setIsEditDialogOpen(true);
  };

  const handleDelete = (event: Event) => {
    event.preventDefault();
    setIsDeleteDialogOpen(true);
  };

  const handleConfirmDelete = () => {
    if (type === "part") {
      deletePartMutation.mutate(id);
    } else {
      deleteExhibitMutation.mutate(id);
    }
    setIsDeleteDialogOpen(false);
  };

  return (
    <DropdownMenu modal={false}>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" size="icon" className="h-8 w-8">
          <MoreVertical className="h-4 w-4" />
          <span className="sr-only">More actions</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem onSelect={handleEdit}>
          <Pencil className="mr-2 h-4 w-4" />
          Edit
        </DropdownMenuItem>
        <DropdownMenuItem onSelect={handleDelete} className="text-destructive">
          <Trash2 className="mr-2 h-4 w-4" />
          Delete
        </DropdownMenuItem>
      </DropdownMenuContent>

      <Dialog open={isEditDialogOpen} onOpenChange={setIsEditDialogOpen}>
        <GenericEditDialog
          id={id}
          type={type}
          isOpen={isEditDialogOpen}
          onClose={() => setIsEditDialogOpen(false)}
        />
      </Dialog>

      <Dialog open={isDeleteDialogOpen} onOpenChange={setIsDeleteDialogOpen}>
        <DeletionConfirmationDialog
          isOpen={isDeleteDialogOpen}
          onClose={() => setIsDeleteDialogOpen(false)}
          onConfirm={handleConfirmDelete}
          itemType={type}
        />
      </Dialog>
    </DropdownMenu>
  );
}

interface DeletionConfirmationDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onConfirm: () => void;
  itemType: "exhibit" | "part";
}

export function DeletionConfirmationDialog({
  isOpen,
  onClose,
  onConfirm,
  itemType,
}: DeletionConfirmationDialogProps) {
  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-background/80 backdrop-blur-xs flex items-center justify-center z-50">
      <div
        className="bg-background border border-border rounded-lg shadow-lg p-6 max-w-sm w-full"
        role="dialog"
        aria-modal="true"
        aria-labelledby="dialog-title"
      >
        <h2
          id="dialog-title"
          className="text-xl font-bold mb-4 text-foreground"
        >
          Confirm Deletion
        </h2>
        <p className="mb-6 text-muted-foreground">
          Are you sure you want to delete this {itemType}? This action cannot be
          undone.
        </p>
        <div className="flex justify-end space-x-4">
          <Button variant="outline" onClick={onClose}>
            Cancel
          </Button>
          <Button variant="destructive" onClick={onConfirm}>
            Delete
          </Button>
        </div>
      </div>
    </div>
  );
}
