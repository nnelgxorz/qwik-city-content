import { component$ } from '@builder.io/qwik';
import type { DocumentHead } from '@builder.io/qwik-city';

export default component$(() => {
  return (
    <div>
      <h2>Welcome to Qwik City Blog</h2>

      <p>The blog meta-framework for Qwik.</p>

      <h3>
        <p>Recent Posts</p>
        <ul>

        </ul>
      </h3>
    </div>
  );
});

export const head: DocumentHead = {
  title: 'Welcome to Qwik City',
};
