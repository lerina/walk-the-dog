import init, { World, Circle } from "../pkg/mv_circle.js";

async function run() {
    const wasm = await init().catch(console.error);
    //await init();

    let canvas = document.getElementById("canvas");
    let ctx = canvas.getContext('2d');
    const world = new World(600, 600);
    let circle = world.circle;
    let sine_circle = new Circle(10.0, 300.0, 10.0, 0.0, 2.0 * Math.PI);

    function keyDown(event) {
            //console.log("anim On: "+ `key=${event.key},code=${event.code}`);
            switch (event.keyCode) {
                case 37:
                    //'Left Key pressed!';
                    sine_circle.mv_left(-2.5, 10.0);
                    console.log("y "+ sine_circle.center_y);
                    break;
                case 38:
                    //'Up Key pressed!';
                    sine_circle.mv_up(-2.5, 10.0);
                    console.log("x "+ sine_circle.center_x);
                    event.detail.keyboardEvent.preventDefault();
                    break;
                case 39:
                    //'Right Key pressed!';
                    sine_circle.mv_right(2.5, 590.0);
                    console.log("y "+ sine_circle.center_y);
                    break;
                case 40:
                    //'Down Key pressed!';
                    sine_circle.mv_down(2.5, 590.0);
                    console.log("x "+ sine_circle.center_x);
                    // https://github.com/PolymerElements/iron-a11y-keys-behavior/issues/13
                    event.detail.keyboardEvent.preventDefault();
                    break;
            }
    }
    function keyUp(event) {
            //console.log("anim Off: "+ `key=${event.key},code=${event.code}`);
    }

    window.addEventListener('keydown', keyDown);
    window.addEventListener('keyup', keyUp);

    

    function draw_circle() {
      //circle = world.circle;

      ctx.beginPath();
      ctx.arc(circle.center_x, circle.center_y, circle.radius, 
              circle.start_angle, circle.end_angle);
      ctx.arc(sine_circle.center_x, sine_circle.center_y, sine_circle.radius, 
              sine_circle.start_angle, sine_circle.end_angle);
      ctx.closePath();

      ctx.fillStyle = world.circle_color; //'rgba(177, 0, 129, .1)';
      ctx.fill();

      ctx.lineWidth = 3;
      ctx.strokeStyle = '#003300';
      ctx.stroke();

    }

    function draw() {
	    ctx.clearRect(0, 0, world.width, world.height);
	    ctx.fillStyle = '#F6F6F6';
	    ctx.fillRect(0, 0, world.width, world.height);

        draw_circle();
        //sine_circle.update();
        
	    // call the draw function again!
	    requestAnimationFrame(draw);
    }
requestAnimationFrame(draw);
}

run();
