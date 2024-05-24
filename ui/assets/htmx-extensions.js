/*
 * Zero-Clause BSD
 * =============
 *
 * Permission to use, copy, modify, and/or distribute this software for
 * any purpose with or without fee is hereby granted.
 *
 * THE SOFTWARE IS PROVIDED “AS IS” AND THE AUTHOR DISCLAIMS ALL
 * WARRANTIES WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES
 * OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE
 * FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY
 * DAMAGES WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN
 * AN ACTION OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT
 * OF OR IN CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.
 *
 */

/*
 * The HTMX extensions don't have ES6 module support (though HTMX itself does),
 * so any extensions we need can be copied here (they're usually very small),
 * wrapped in a function, and exported.
 */

// Remove Me Extension: https://htmx.org/extensions/remove-me/
export const removeMe = htmx => {
  function maybeRemoveMe (elt) {
    const timing = elt.getAttribute('remove-me') || elt.getAttribute('data-remove-me')
    if (timing) {
      setTimeout(function () {
        if (elt && elt.parentElement) {
          elt.parentElement.removeChild(elt)
        }
      }, htmx.parseInterval(timing))
    }
  }
  htmx.defineExtension('remove-me', {
    onEvent: function (name, evt) {
      if (name === 'htmx:afterProcessNode') {
        const elt = evt.detail.elt
        if (elt.getAttribute) {
          maybeRemoveMe(elt)
          if (elt.querySelectorAll) {
            const children = elt.querySelectorAll('[remove-me], [data-remove-me]')
            for (let i = 0; i < children.length; i++) {
              maybeRemoveMe(children[i])
            }
          }
        }
      }
    }
  })
}
