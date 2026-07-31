import "../../chunks/runtime.js";
function _layout($$renderer, $$props) {
  $$renderer.component(($$renderer2) => {
    let { children } = $$props;
    $$renderer2.push(`<div class="app svelte-12qhfyh">`);
    children($$renderer2);
    $$renderer2.push(`<!----></div>`);
  });
}
export {
  _layout as default
};
