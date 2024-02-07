// Global DOM selector helpers
export const $ = selector => document.querySelector(selector)
export const $$ = selector => document.querySelectorAll(selector)

// Will interrogate for input types for a given <form> and attempt to massage
// them into basic JSON types.
// A data attr named data-enctype="TYPE" can be used on any element with a name
// attr for type hinting (e.g. for <select> elements that have no type attr)
export const encodeFormParameters = ($targets, props = {}) => {
  $targets.forEach($element => {
    let value
    switch (($element.dataset && $element.dataset.encType) || $element.type) {
      case 'number':
        try {
          value = parseInt($element.value, 10)
        } catch (e) {
          console.error(`Failed parsing number for "${$element.name}"`,
            $element.name, e)
          value = 0
        }
        break
      case 'boolean':
        if ($element.type === 'checkbox' && $element.checked) {
          value = $element.checked
        } else {
          switch ($element.value.toLowerCase()) {
            case 'yes':
            case 'true':
            case 'on':
              value = true
              break
          }
        }
        break
      case 'checkbox':
        if ($element.checked) {
          value = $element.checked
        }
        break
      case 'text':
      default:
        value = $element.value
        break
    }
    if (value) {
      props[$element.name] = value
    } else {
      delete props[$element.name]
    }
  })
  return props
}
