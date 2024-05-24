let elem = document.getElementById("data")

function update() {
    const needle = document.getElementById("search").value

    for (const cls of globalIndex.classes) {
        const sub = document.getElementById(`c-${cls}`)
        sub.hidden = !cls.includes(needle)
    }

    for (const mName in globalIndex.methods) {
        const mData = globalIndex.methods[mName]
        const sub = document.getElementById(`m-${mName}-c-${mData.c}`)
        sub.hidden = !mName.includes(needle)
    }

    for (const fName in globalIndex.fields) {
        const fData = globalIndex.fields[fName]
        const sub = document.getElementById(`f-${fName}-c-${fData.c}`)
        sub.hidden = !fName.includes(needle)
    }
}

{
    const needle = document.getElementById("search").value
    const subElemType = "li"

    for (const cls of globalIndex.classes) {
        let link = document.createElement("a")
        link.setAttribute("href", `class/${cls}.html`)
        link.setAttribute("target", "_blank")
        link.innerText = cls

        let sub = document.createElement("p")
        sub.setAttribute("id", `c-${cls}`)
        sub.setAttribute("class", "g")
        sub.hidden = !cls.includes(needle)
        sub.innerText = "[class] "
        sub.appendChild(link)
        elem.appendChild(sub)
    }

    for (const mName in globalIndex.methods) {
        const mData = globalIndex.methods[mName]

        let link = document.createElement("a")
        let href = `class/${mData.c}.html#m-${mName}`
        link.setAttribute("href", href)
        link.setAttribute("target", "_blank")
        link.innerText = mName

        let sub = document.createElement(subElemType)
        sub.setAttribute("id", `m-${mName}-c-${mData.c}`)
        sub.setAttribute("class", "g")
        sub.hidden = !mName.includes(needle)
        sub.innerHTML = "[method] "
        sub.appendChild(link)
        sub.innerHTML += " of " + mData.c
        if (!mData.o) sub.innerHTML += " (inherited)"
        elem.appendChild(sub)
    }

    for (const fName in globalIndex.fields) {
        const fData = globalIndex.fields[fName]

        let link = document.createElement("a")
        let href = `class/${fData.c}.html#f-${fName}`
        link.setAttribute("href", href)
        link.setAttribute("target", "_blank")
        link.innerText = fName

        let sub = document.createElement(subElemType)
        sub.setAttribute("id", `f-${fName}-c-${fData.c}`)
        sub.setAttribute("class", "g")
        sub.hidden = !fName.includes(needle)
        sub.innerText = "[field] "
        sub.appendChild(link)
        sub.innerHTML += " of " + fData.c
        if (!fData.o) sub.innerHTML += " (inherited)"
        sub.innerHTML += "<br />"
        elem.appendChild(sub)
    }
}
