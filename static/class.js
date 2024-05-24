"use strict"

{
    const parts = location.href.split("#")
    const id = parts[parts.length - 1]
    const elem = document.getElementById(id)
    if (elem !== null) elem.parentElement.open = true
}
