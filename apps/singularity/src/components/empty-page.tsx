import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyTitle,
} from "@/components/ui/empty";

export function EmptyPage({}: React.ComponentProps<"div">) {
  return (
    <Empty>
      <EmptyHeader>
        <EmptyTitle>404 - Not Found</EmptyTitle>
        <EmptyDescription>
          The page you're looking for doesn't exist. It's probably not
          implemented yet.
        </EmptyDescription>
      </EmptyHeader>
    </Empty>
  );
}
