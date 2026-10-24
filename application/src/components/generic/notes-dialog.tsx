import { useState, useRef, useEffect } from "react";
import { compareDesc, format } from "date-fns";
import {
  StickyNote,
  Loader2,
  Trash2,
  Calendar,
  User,
  Plus,
  Search,
  X,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardFooter,
} from "@/components/ui/card";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import type { Note } from "@/types";
import { NoteForm } from "@/components/forms/create-note-form";
import { Input } from "@/components/ui/input";

interface NotesDialogProps<T extends "part" | "exhibit"> {
  id: string;
  type: T;
  name?: string;
  notes: Note[];
  onNoteAdded?: () => void;
  onNoteDeleted?: () => void;
  createNote: (data: {
    id: string;
    note: { submitter: string; message: string };
  }) => Promise<void>;
  deleteNote: (data: { id: string; noteId: string }) => Promise<void>;
}

export function NotesDialog<T extends "part" | "exhibit">({
  id,
  type,
  name,
  notes,
  onNoteAdded,
  onNoteDeleted,
  createNote,
  deleteNote,
}: NotesDialogProps<T>) {
  const [isOpen, setIsOpen] = useState(false);
  const [isDeleting, setIsDeleting] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const [isAddingNote, setIsAddingNote] = useState(false);
  const [cardHeight, setCardHeight] = useState(0);
  const cardRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (cardRef.current) {
      setCardHeight(cardRef.current.scrollHeight);
    }
  }, [isAddingNote]);

  const handleDelete = async (noteId: string) => {
    setIsDeleting(true);
    try {
      await deleteNote({ id, noteId });
      if (onNoteDeleted) {
        onNoteDeleted();
      }
    } catch (error) {
      console.error("Error deleting note:", error);
    } finally {
      setIsDeleting(false);
    }
  };

  const handleNoteAdded = () => {
    if (onNoteAdded) {
      onNoteAdded();
    }
    setIsAddingNote(false);
  };

  const filteredNotes = notes.filter(
    (note) =>
      note.submitter.toLowerCase().includes(searchQuery.toLowerCase()) ||
      note.message.toLowerCase().includes(searchQuery.toLowerCase())
  );

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button
          variant="outline"
          size="sm"
          className="transition-all duration-200 hover:bg-secondary"
        >
          <StickyNote className="w-4 h-4 mr-2" />
          <span>Notes</span>
          <Badge variant="secondary" className="ml-2">
            {notes.length}
          </Badge>
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <DialogTitle className="text-2xl font-semibold">
            {name || `${type.charAt(0).toUpperCase() + type.slice(1)} Notes`}
          </DialogTitle>
        </DialogHeader>
        <div className="mt-4 space-y-4">
          <div className="flex items-center space-x-2">
            <div className="relative grow">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 text-muted-foreground w-4 h-4" />
              <Input
                type="text"
                placeholder="Search notes..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="pl-10 w-full"
              />
            </div>
            <Button
              onClick={() => setIsAddingNote(true)}
              variant="outline"
              size="sm"
              className="px-2"
            >
              <Plus className="h-4 w-4" />
              <span className="sr-only">Add Note</span>
            </Button>
          </div>
          <div
            style={{
              maxHeight: isAddingNote ? `${cardHeight}px` : "0px",
              opacity: isAddingNote ? 1 : 0,
              transform: `translateY(${isAddingNote ? "0" : "-20px"})`,
            }}
            className="transition-all duration-700 ease-out overflow-hidden"
          >
            <Card className="w-full" ref={cardRef}>
              <CardHeader className="flex flex-col space-y-1.5 pb-3">
                <div className="flex justify-between items-center">
                  <h3 className="text-lg font-semibold">Add a new note</h3>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setIsAddingNote(false)}
                    className="h-8 w-8 p-0"
                  >
                    <X className="h-4 w-4" />
                    <span className="sr-only">Close</span>
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                <NoteForm
                  id={id}
                  onSuccess={handleNoteAdded}
                  createNote={createNote}
                />
              </CardContent>
            </Card>
          </div>
          <NotesList
            notes={filteredNotes}
            handleDelete={handleDelete}
            isDeleting={isDeleting}
          />
        </div>
      </DialogContent>
    </Dialog>
  );
}

interface NotesListProps {
  notes: Note[];
  handleDelete: (noteId: string) => void;
  isDeleting: boolean;
}

function NotesList({ notes, handleDelete, isDeleting }: NotesListProps) {
  return (
    <ScrollArea className="h-[60vh] pr-4">
      {notes.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-full text-center">
          <StickyNote className="w-12 h-12 text-muted-foreground mb-2" />
          <p className="text-muted-foreground">No notes found</p>
          <p className="text-sm text-muted-foreground">
            Try adjusting your search or add a new note
          </p>
        </div>
      ) : (
        <div className="space-y-4">
          {notes
            .sort((a, b) =>
              compareDesc(
                new Date(a.timestamp.date + " " + a.timestamp.time),
                new Date(b.timestamp.date + " " + b.timestamp.time)
              )
            )
            .map((note) => (
              <NoteCard
                key={note.id}
                note={note}
                handleDelete={handleDelete}
                isDeleting={isDeleting}
              />
            ))}
        </div>
      )}
    </ScrollArea>
  );
}

interface NoteCardProps {
  note: Note;
  handleDelete: (noteId: string) => void;
  isDeleting: boolean;
}

function NoteCard({ note, handleDelete, isDeleting }: NoteCardProps) {
  const [showConfirm, setShowConfirm] = useState(false);
  const formattedDate = format(
    new Date(note.timestamp.date + " " + note.timestamp.time),
    "PPpp"
  );

  const onDelete = () => {
    setShowConfirm(true);
  };

  const confirmDelete = () => {
    handleDelete(note.id);
    setShowConfirm(false);
  };

  const cancelDelete = () => {
    setShowConfirm(false);
  };

  return (
    <Card className="w-full hover:shadow-md transition-shadow duration-300">
      <CardHeader className="pb-2">
        <div className="flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <User className="w-4 h-4 text-primary" />
            <p className="text-sm font-medium">{note.submitter}</p>
          </div>
          <div className="flex items-center space-x-2">
            <Calendar className="w-4 h-4 text-muted-foreground" />
            <p className="text-xs text-muted-foreground">{formattedDate}</p>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <p className="text-sm leading-relaxed whitespace-pre-wrap">
          {note.message}
        </p>
      </CardContent>
      <CardFooter className="pt-2">
        <div className="flex justify-end w-full relative h-10 overflow-hidden">
          <div
            className={`
              absolute inset-y-0 right-0 flex items-center
              transition-all duration-500 ease-in-out
              ${
                showConfirm
                  ? "translate-y-full opacity-0"
                  : "translate-y-0 opacity-100"
              }
            `}
          >
            <Button
              variant="ghost"
              size="sm"
              onClick={onDelete}
              className="text-muted-foreground hover:text-destructive transition-colors duration-200"
            >
              <Trash2 className="w-4 h-4" />
              <span className="sr-only">Delete note</span>
            </Button>
          </div>
          <div
            className={`
              absolute inset-y-0 right-0 flex items-center space-x-2
              transition-all duration-500 ease-in-out
              ${
                showConfirm
                  ? "translate-y-0 opacity-100"
                  : "-translate-y-full opacity-0"
              }
            `}
          >
            <Button
              variant="destructive"
              size="sm"
              onClick={confirmDelete}
              disabled={isDeleting}
            >
              {isDeleting ? (
                <Loader2 className="w-4 h-4 animate-spin" />
              ) : (
                "Confirm"
              )}
            </Button>
            <Button variant="outline" size="sm" onClick={cancelDelete}>
              Cancel
            </Button>
          </div>
        </div>
      </CardFooter>
    </Card>
  );
}
