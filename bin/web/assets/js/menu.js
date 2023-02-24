let button = document.querySelector('#mobile-menu-button');
button.addEventListener('click', function() {
    let menu = document.querySelector('#mobile-menu');
    menu.classList.toggle('hidden');
    let closed = menu.classList.contains('hidden');
    let hamburger = document.querySelector('#mobile-menu-hamburger');
    let close = document.querySelector('#mobile-menu-close');
    if (closed) {
        hamburger.classList.remove('hidden');
        hamburger.classList.add('block');
        close.classList.add('hidden');
        close.classList.remove('block');
    } else {
        hamburger.classList.add('hidden');
        hamburger.classList.remove('block');
        close.classList.remove('hidden');
        close.classList.add('block');
    }
});

let current = '/' + location.pathname.split("/")[1];
let nav = document.getElementsByClassName("menu-links");
for (let n = 0; n < nav.length; n++) {
    let items = nav[n].getElementsByTagName("a");
    for (let i = 0; i < items.length; i++) {
        if (items[i].getAttribute("href") === current) {
            items[i].classList.add("bg-gray-900","text-white");
        } else {
            items[i].classList.add("text-gray-300","hover:bg-gray-700","hover:text-white")
        }
    }
}
