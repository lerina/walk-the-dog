import init from "../pkg/walk_the_dog.js";

async function run() {
    const wasm = await init(); //.catch(console.error);
    const memory = wasm.memory;

}//^--run

//-------------------
run();
