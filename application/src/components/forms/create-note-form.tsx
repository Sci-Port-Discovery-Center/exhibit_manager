import { useState } from "react";
import { useForm } from "react-hook-form";
import { Loader2, Plus } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { currentUser } from "@/lib/current-user";

interface NoteFormProps {
  id: string;
  onSuccess: () => void;
  createNote: (data: {
    id: string;
    note: { submitter: string; message: string };
  }) => Promise<void>;
}

export function NoteForm({ id, onSuccess, createNote }: NoteFormProps) {
  const { register, handleSubmit, reset } = useForm<{ message: string }>();
  const [isSubmitting, setIsSubmitting] = useState(false);

  const onSubmit = async (formData: { message: string }) => {
    setIsSubmitting(true);
    try {
      await createNote({
        id,
        note: { submitter: currentUser.name, message: formData.message },
      });
      reset();
      onSuccess();
    } catch (error) {
      console.error("Error creating note:", error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <form onSubmit={handleSubmit(onSubmit)} className="flex space-x-2">
      <Input
        {...register("message", { required: true })}
        placeholder="Add a new note..."
        className="grow"
      />
      <Button
        type="submit"
        disabled={isSubmitting}
        variant="outline"
      >
        {isSubmitting ? (
          <Loader2 className="w-4 h-4 animate-spin" />
        ) : (
          <Plus className="h-4 w-4" />
        )}
      </Button>
    </form>
  );
}
