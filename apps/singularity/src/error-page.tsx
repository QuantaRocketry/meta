import { useRouteError } from "react-router-dom";
import Page from "./utils/page";
import { EmptyPage } from "./components/empty-page";

export default function ErrorPage() {
  const error: any = useRouteError();
  console.error(error);

  return (
    <Page title={"Error"} className="h-full w-full items-center content-center">
      <EmptyPage />
    </Page>
  );
}
