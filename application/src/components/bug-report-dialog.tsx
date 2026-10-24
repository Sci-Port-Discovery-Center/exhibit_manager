import * as React from "react";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { Bug } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useMutation } from "@tanstack/react-query";
import { currentUser } from "@/lib/current-user";
import { axiosInstance } from "@/api/axiosInstance";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Button } from "@/components/ui/button";
import { SidebarMenuButton, SidebarMenuItem } from "@/components/ui/sidebar";

const formSchema = z.object({
  title: z.string().min(2, {
    message: "Title must be at least 2 characters.",
  }),
  description: z.string().min(10, {
    message: "Description must be at least 10 characters.",
  }),
});

type FormSchema = z.infer<typeof formSchema>;

interface BugReportPayload extends FormSchema {
  name?: string;
}

async function reportBug(bug_report: BugReportPayload) {
  await axiosInstance.post("/report-bug", bug_report);
}

export function BugReportDialog() {
  const [isOpen, setIsOpen] = React.useState(false);
  const { toast } = useToast();

  const form = useForm<FormSchema>({
    resolver: zodResolver(formSchema),
    defaultValues: {
      title: "",
      description: "",
    },
  });

  const bugReportMutation = useMutation({
    mutationKey: ["reportBug"],
    mutationFn: async (values: FormSchema) => {
      const enrichedBugReport = {
        ...values,
        name: currentUser.name,
      };
      return await reportBug(enrichedBugReport);
    },
  });

  const onSubmit = (values: FormSchema) => {
    bugReportMutation.mutate(values, {
      onSuccess: () => {
        toast({
          title: "Bug report submitted",
          description: "Thank you for your feedback!",
        });
        form.reset();
        setIsOpen(false);
      },
      onError: (error) => {
        toast({
          title: "Error",
          description:
            error instanceof Error
              ? error.message
              : "Failed to submit bug report. Please try again.",
          variant: "destructive",
        });
      },
    });
  };

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <SidebarMenuItem>
          <SidebarMenuButton
            asChild
            size="sm"
            className="text-muted-foreground hover:text-foreground"
          >
            <button onClick={() => setIsOpen(true)}>
              <Bug className="h-4 w-4" />
              <span>Report a bug</span>
            </button>
          </SidebarMenuButton>
        </SidebarMenuItem>
      </DialogTrigger>
      <DialogContent className="sm:max-w-[425px]">
        <DialogHeader>
          <DialogTitle>Report a Bug</DialogTitle>
          <DialogDescription>
            Describe the issue you're experiencing. I'll look into it as soon as
            possible.
          </DialogDescription>
        </DialogHeader>
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-8">
            <FormField
              control={form.control}
              name="title"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Subject</FormLabel>
                  <FormControl>
                    <Input placeholder="Can't login" {...field} />
                  </FormControl>
                  <FormDescription>
                    Provide a short, descriptive subject for the bug.
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Description</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="Detailed description of the bug, including steps to reproduce if possible"
                      className="resize-none"
                      {...field}
                    />
                  </FormControl>
                  <FormDescription>
                    Provide as much detail as possible about the bug.
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button type="submit" disabled={bugReportMutation.isPending}>
              {bugReportMutation.isPending
                ? "Submitting..."
                : "Submit Bug Report"}
            </Button>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
