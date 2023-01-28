import { A } from "@solidjs/router";
import { createEffect, createSignal, For, Suspense } from "solid-js";
import Loading from "./Loading";

export default function Pagination(props: any) {
  let pageCount = 0;
  let beforeOffset = 0;
  let left = 0;
  let right = 0;
  let afterOffset = 0;
  const [pageArray, setPageArray] = createSignal<Array<number>>();

  createEffect(() => {
    let total: number = props.pagination?.total | 0;
    let page_size: number = props.pagination?.page_size | 0;
    let current = props.pagination?.current | 0;
    pageCount = Math.ceil(total / page_size);
    beforeOffset = current - 5;

    left = Math.max(1, beforeOffset);
    right = Math.min(current + 4, pageCount);

    afterOffset = pageCount - current - 5;

    if (beforeOffset < 0) {
      right = Math.min(pageCount, right - beforeOffset);
    }

    if (afterOffset < 0) {
      left = Math.max(1, afterOffset + left);
    }
    const pageArray = new Array<number>();
    for (let i = left; i <= right; i++) {
      pageArray.push(i);
    }
    setPageArray(pageArray);
  });

  return (
    <div class="flex justify-center py-4 gap-2 select-none">
      <Suspense>
        <section class="flex items-center">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            onClick={() => {
              if (props.pagination?.current <= 1) {
                return;
              }
              props.pageSetter("current", (p) => p - 1);
            }}
            width="1em"
            classList={{
              "fill-gray-200": props.pagination?.current <= 1,
              "hover:fill-blue-500": props.pagination?.current > 1,
              "fill-gray-600": props.pagination?.current > 1,
            }}
            class="cursor-pointer transition"
            height="1em"
            viewBox="0 0 24 24"
          >
            <path d="M10 22L0 12L10 2l1.775 1.775L3.55 12l8.225 8.225Z" />
          </svg>
        </section>
        <section class="flex gap-4">
          <For each={pageArray()}>
            {(item) => (
              <span
                onClick={() => {
                  props.pageSetter("current", item);
                }}
                class="cursor-pointer transition"
                classList={{
                  "text-blue-500": item === props.pagination?.current,
                  "text-gray-600 hover:text-blue-500":
                    item !== props.pagination?.current,
                }}
              >
                {item}
              </span>
            )}
          </For>
        </section>
        <section class="flex items-center">
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="1em"
            height="1em"
            onClick={() => {
              if (props.pagination?.current >= pageCount) {
                return;
              }
              props.pageSetter("current", (p) => p + 1);
            }}
            class="cursor-pointer transition-all"
            classList={{
              "fill-gray-200": props.pagination?.current >= pageCount,
              "hover:fill-blue-500": props.pagination?.current < pageCount,
              "fill-gray-600": props.pagination?.current < pageCount,
            }}
            viewBox="0 0 24 24"
          >
            <path d="M8.025 22L6.25 20.225L14.475 12L6.25 3.775L8.025 2l10 10Z" />
          </svg>
        </section>
      </Suspense>
    </div>
  );
}
