"use strict"

let pageContents = document.getElementById("data")
let statusUI = document.getElementById("status")
let quoteUI = document.getElementById("quote")

const itemsPerPage = 40
let rangeStart = 0
let rangeEnd = itemsPerPage

function resetRange() {
    rangeStart = 0
    rangeEnd = itemsPerPage
    updateStatus()
}

function buildClassElement(name) {
    let link = document.createElement("a")
    link.setAttribute("href", `class/${name}.html`)
    link.setAttribute("target", "_blank")
    link.innerText = name

    let element = document.createElement("li")
    element.setAttribute("class", "g")
    element.innerText = "[class] "
    element.appendChild(link)

    return element
}

function buildMemberElement(type, name, data) {
    const typeLetter = type[0]

    let link = document.createElement("a")
    let href = `class/${data.c}.html#${typeLetter}-${name}`
    link.setAttribute("href", href)
    link.setAttribute("target", "_blank")
    link.innerText = name

    let element = document.createElement("li")
    element.setAttribute("class", "g")
    element.innerHTML = `[${type}] `
    element.appendChild(link)
    element.innerHTML += " of " + data.c
    if (!data.o) element.innerHTML += " (inherited)"

    return element
}

function updateStatus() {
    statusUI.innerText = `${rangeStart} to ${rangeEnd}`
}

function updatePage() {
    const needles = document.getElementById("search").value.split(" ")
    
    // Clear
    pageContents.innerText = ""

    let count = 0

    function found(builder) {
        if (rangeStart <= count && count < rangeEnd) {
            pageContents.appendChild(builder())
        }

        count++
    }

    function hasNeedles(name) {
        for (const needle of needles) {
            if (!name.includes(needle)) {
                return false
            }
        }

        return true
    }

    for (const cls of globalIndex.classes) {
        if (hasNeedles(cls)) {
            found(() => buildClassElement(cls))
            if (count === rangeEnd) return
        }
    }

    for (const mName in globalIndex.methods) {
        if (hasNeedles(mName)) {
            const mData = globalIndex.methods[mName]
            found(() => buildMemberElement("method", mName, mData))
            if (count === rangeEnd) return
        }
    }

    for (const fName in globalIndex.fields) {
        if (hasNeedles(fName)) {
            const fData = globalIndex.fields[fName]
            found(() => buildMemberElement("field", fName, fData))
            if (count === rangeEnd) return
        }
    }
}

function pageChange(increment) {
    rangeStart += increment
    rangeEnd += increment

    if (rangeStart < 0) resetRange()

    updateStatus()
    updatePage()
}

function pageNext() {
    pageChange(itemsPerPage)
}

function pagePrevious() {
    pageChange(-itemsPerPage)
}

function onSearchEdited() {
    resetRange()
    updatePage()
}

onSearchEdited()
quoteUI.innerHTML = '<abbr title="Quote of The Request">QoTR</abbr>: ' + globalQuoteList[(Math.random() * globalQuoteList.length) | 0]
