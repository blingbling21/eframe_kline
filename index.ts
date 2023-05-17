import { renderWithQiankun, qiankunWindow } from 'vite-plugin-qiankun/dist/helper';

import * as wasm from "./pkg";


renderWithQiankun({
    mount(props) {
        console.log('mount props: ', props);
        init(props.height);
        wasm.main();
    },
    bootstrap() {
        console.log('bootstrap');
    },
    unmount(props) {
        console.log('unmount props: ', props);
    },
    update(props) {
        console.log('update props: ', props);
        init(props.height);
    },
})

if (!qiankunWindow.__POWERED_BY_QIANKUN__) {
    wasm.main();
}


const init = (height: number) => {
    const parent = document.querySelector('.parent') as HTMLDivElement;
    parent.style.height = `${height}px`;
    // wasm.main();
}
