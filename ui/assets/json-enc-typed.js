// Adapted from: https://htmx.org/extensions/json-enc
// Will interrogate input type and try to massage them into basic types
// Can also provide a data-enctype="TYPE" for more hinting (e.g. for <select>)
htmx.defineExtension('json-enc-typed', {
    onEvent: function (name, evt) {
        if (name === 'htmx:configRequest') {
            evt.detail.headers['Content-Type'] = 'application/json'
        }
    },
    
    encodeParameters : function(xhr, parameters, elt) {
        elt.querySelectorAll('[name]').forEach(element => {
            console.log('element before:', element.name, element.value);
            let value = element.value
            switch(element.dataset && element.dataset.encType || element.type) {
                case 'number':
                    try {
                        value = parseInt(element.value, 10)
                    } catch (e) {
                        console.error('failed parsing element', element.name, e)
                        value = 0
                    }
                break
                case 'boolean':
                    value = false
                    if (element.type === 'checkbox') {
                        value = element.checked
                    } else {
                        switch (element.value.toLowerCase()) {
                            case 'yes':
                            case 'true':
                                value = true
                            break
                        }
                    }
                break
                default:
                    // falls through
                case 'text':
                    value = element.value
                break
            }
            parameters[element.name] = value
        })
        xhr.overrideMimeType('text/json')
        return JSON.stringify(parameters)
    }
})