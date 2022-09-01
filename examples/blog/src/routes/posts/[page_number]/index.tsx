import { component$, Resource } from "@builder.io/qwik";
import { RequestHandler, useEndpoint } from "@builder.io/qwik-city";
import Testimonial from "../../../components/testimonial"
import { RouteParams } from "./generated";

export default component$(() => {
  const content = useEndpoint<typeof onGet>();
  return <Resource value={content}
    onResolved={({ post, testimonial }) => {
      return <>
        <article>

        </article>
        {testimonial && <Testimonial testimonial={testimonial} />}
      </>
    }
    }
  />
})

export type Content = {
  post: any
  testimonial: any | undefined
}

export const onGet: RequestHandler<Content> = ({ params }) => {
  let { id } = params as RouteParams;
  return { post: undefined, testimonial: undefined }
}