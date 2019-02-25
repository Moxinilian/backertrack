import { Elm } from './Main.elm'
import domready from 'domready'

domready(() => {
    const node = document.querySelector('body')
    Elm.Main.init({
        node, flags: {
            apiServer: process.env.API_URL
        }
    })
})