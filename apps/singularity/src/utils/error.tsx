import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { useEffect, useState } from "react";

export default function ErrorProvider() {
  const [open, setOpen] = useState(false);
  const [message, setMessage] = useState<string | JSX.Element | undefined>(
    undefined,
  );

  useEffect(() => {
    const handleOpen = (e: any) => {
      setMessage(e.detail);
      setOpen(true);
    };

    window.addEventListener("open-global-error", handleOpen);
    return () => window.removeEventListener("open-global-error", handleOpen);
  }, []);

  return (
    <AlertDialog open={open} onOpenChange={setOpen}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>An Error Occurred</AlertDialogTitle>
          <AlertDialogDescription>
            {message || "An unexpected error has occurred."}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogAction onClick={() => setOpen(false)}>
            Dismiss
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}

export const showError = (message: string | JSX.Element) => {
  const event = new CustomEvent("open-global-error", { detail: message });
  window.dispatchEvent(event);
};
