import { component$, useStore } from "@builder.io/qwik";

export default component$(() => {
  const store = useStore({
    count: 0
  })
  return <button onClick$={() => store.count += 1}>The count is {store.count}</button>
}) 