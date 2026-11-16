import { SerialControlWidget } from "@/components/header-widgets/serial-control";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { showError } from "@/utils/error";
import Page from "@/utils/page";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";

function SerialStream() {
  const [content, setContent] = useState([""]);
  const [inputMessage, setInputMessage] = useState("");

  function appendContent(s: string) {
    setContent((prev) => [...prev, s]);
  }

  async function sendSerialMessage(s: string) {
    setInputMessage("");
    invoke("send_serial_message", { message: s })
      .then((_) => {
        appendContent(s);
      })
      .catch((e) => {
        showError(e);
      });
  }

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    async function serialListener() {
      unlisten = await listen<string>("serial_message_received", () => {
        invoke("get_serial_content").then((c) => {
          setContent(c as string[]);
        });
      });
    }

    serialListener();
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    invoke("get_serial_content").then((c) => {
      setContent(c as string[]);
    });
  }, []);

  const contentRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    if (contentRef.current) {
      contentRef.current.scrollTop = contentRef.current.scrollHeight;
    }
  };

  useEffect(() => {
    scrollToBottom();
  }, [content]);

  return (
    <Card className="flex flex-col h-full min-h-0 grow">
      <CardContent className="flex flex-col flex-1 min-h-0 gap-4">
        {/* The main scrollable area */}
        <div
          id="serial-content"
          ref={contentRef}
          className="flex-1 overflow-y-auto border rounded-md p-2 bg-slate-50 dark:bg-slate-900 font-mono text-sm"
        >
          {content.map((line, index) => (
            <div key={index} className="whitespace-pre-wrap">
              {line}
            </div>
          ))}
        </div>

        <div>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              const form = e.currentTarget;
              const input = form.elements.namedItem(
                "serialInput",
              ) as HTMLInputElement;
              sendSerialMessage(input.value);
            }}
            className="flex w-full gap-2"
          >
            <Input
              id="serialInput"
              value={inputMessage}
              onChange={(e) => setInputMessage(e.target.value)}
              placeholder="Send a message..."
              className="flex-1"
            />
            <Button type="submit">Send</Button>
          </form>
        </div>
      </CardContent>
    </Card>
  );
}

export default function Terminal() {
  return (
    <Page title="Terminal" widgets={[<SerialControlWidget />]}>
      <SerialStream />
    </Page>
  );
}
