import { component$, useStyles$ } from '@builder.io/qwik';
import { useLocation } from '@builder.io/qwik-city';
import styles from './header.css?inline';

export default component$(() => {
  useStyles$(styles);

  const pathname = useLocation().pathname;

  return (
    <header>
      <div class="header-inner">
        <section class="logo">
          <a href="/">Qwik City Blog 🏙</a>
        </section>
        <nav>
          <a href="/posts" class={{ active: pathname.startsWith('/posts') }}>
            Blog
          </a>
        </nav>
      </div>
    </header>
  );
});
