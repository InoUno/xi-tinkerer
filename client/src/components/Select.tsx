import {
  Component,
  For,
  JSX,
  Setter,
  Show,
  createEffect,
  createMemo,
  createSignal,
  on,
} from "solid-js";

const Select: Component<{
  setSelection: Setter<string>;
  options: Array<string>;
  autofocus?: boolean;
}> = (props) => {
  const [text, setText] = createSignal("");
  const [show, setShow] = createSignal(false);
  const [selected, setSelected] = createSignal(0);

  let inputRef: HTMLInputElement = undefined!;

  const filteredOptions = () =>
    props.options.filter((el) => el.includes(text()));

  const areOptionsVisible = createMemo(() => {
    return (
      show() &&
      (filteredOptions().length > 1 || filteredOptions()[0] !== text())
    );
  });

  createEffect(
    on(text, () => {
      setSelected(0);
    })
  );

  createEffect(() => {
    inputRef?.focus();
  });

  const handleInput: JSX.EventHandlerUnion<HTMLInputElement, InputEvent> = (
    event
  ) => {
    setText(event.currentTarget.value);
  };

  const handleKeydown: JSX.EventHandler<HTMLInputElement, KeyboardEvent> = (
    event
  ) => {
    if (event.code === "ArrowUp") {
      setSelected((prev) =>
        prev === 0 ? filteredOptions().length - 1 : prev - 1
      );
    } else if (event.code === "ArrowDown") {
      setSelected((prev) =>
        prev + 1 === filteredOptions().length ? 0 : prev + 1
      );
    } else if (event.code === "Enter") {
      setText(filteredOptions()[selected()]);
      props.setSelection(text());
    }
  };

  return (
    <div class="flex flex-col relative my-2">
      <input
        ref={inputRef}
        class="block py-1 px-2 bg-slate-500 border border-slate-400 focus:border-slate-200 focus:outline-none rounded-md placeholder:text-slate-400"
        placeholder="Select a zone"
        type="text"
        value={text()}
        onInput={handleInput}
        onKeyDown={handleKeydown}
        onFocus={() => setShow(true)}
        onBlur={() => setShow(false)}
      />
      <Show when={areOptionsVisible()}>
        <div class="relative z-10 bg-slate-600 divide-y divide-gray-100 rounded-md shadow my-1 overflow-y-visible w-full">
          <ul class="absolute flex flex-col w-full top-full bg-slate-600 rounded+md">
            <For each={filteredOptions()}>
              {(item, i) => (
                <li
                  class="py-1 px-2 rounded-md"
                  classList={{
                    "bg-slate-500 text-black": selected() === i(),
                  }}
                >
                  {item}
                </li>
              )}
            </For>
          </ul>
        </div>
      </Show>
    </div>
  );
};

export default Select;
