import { component$, Resource } from "@builder.io/qwik";
import { RequestHandler, useEndpoint } from "@builder.io/qwik-city";
import * as Taxonomies from "@content/taxonomies";
import { RouteParams } from "./generated";

export default component$(() => {
  const data = useEndpoint<typeof onGet>();
  return <Resource value={data} onResolved={(page) => {
    return <article>
      <header>
        <h1>{page.title}</h1>
        {!!page.description && <p>{page.description}</p>}
      </header>
      <div class="content" dangerouslySetInnerHTML={page._html} />

      <h2 id="tags">Tags</h2>
      <ul aria-labelledby="tags">
        {page.tags.map(tag => <a href={`/tags/${tag}`}>{tag}</a>)}
      </ul>
    </article>
  }}
  />
})

export const onGet: RequestHandler<Taxonomies.Posts> = ({ params, response }) => {
  let { id } = params as RouteParams;
  let post = Taxonomies.posts.find(p => p._id == +id);
  if (!post) {
    console.log("No post found")
    throw response.error(404)
  }
  return post
}