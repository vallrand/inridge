class Application {
    static maxDeltaTime = 1000 / 60
    timestamp = 0
    deltaTime = 0
    frame = 0
    systems = []
    register(System, ...options){
        System._uuid = this.systems.push(new System(this, ...options)) - 1
        return this
    }
    fetch(System){ return this.systems[System._uuid] }
    update = timestamp => {
        this.deltaTime = Math.min(Application.maxDeltaTime, timestamp - this.timestamp)
        this.timestamp = timestamp
        this.frame++
        for(let i = 0; i < this.systems.length; i++) this.systems[i].update(this)
        requestAnimationFrame(this.update)
    }
    run(){
        for(let i = 0; i < this.systems.length; i++) if(this.systems[i].start) this.systems[i].start(this)
        this.update(0)
        return this
    }
}

class Renderer {
    constructor(context, { width, height }){
        this.canvas = document.createElement('canvas')
        Object.assign(this.canvas, { width, height })
        this.ctx = this.canvas.getContext('2d', { alpha: false })
    }
    start(context){
        document.body.appendChild(this.canvas)
    }
    update(context){
        this.ctx.resetTransform()
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height)
    }
}

class Camera {
    constructor(context, { x = 0, y = 0 }){ Object.assign(this, { x, y }) }
    update(context){
        const { ctx, canvas } = context.fetch(Renderer)
        ctx.translate(0.5 * canvas.width + this.x, 0.5 * canvas.height + this.y)
    }
}

class Pointer {
    position = new DOMPoint(0, 0)
    frame = 0
    constructor(context){
        this.canvas = context.fetch(Renderer).canvas
        this.canvas.addEventListener('pointermove', event => this.handlePointerEvent(event))
        this.canvas.addEventListener('pointerdown', event => {
            this.handlePointerEvent(event)
            this.frame = 0
            this.pressure = true
        })
        addEventListener('pointerup', event => {
            this.frame = 0
            this.pressure = false
        })
    }
    get pressed(){ return this.pressure }
    get trigger(){ return this.frame <= 1 }
    handlePointerEvent(event){
        const rect = this.canvas.getBoundingClientRect()
        this.position.x = (event.clientX - rect.left) * this.canvas.width / rect.width
        this.position.y = (event.clientY - rect.top) * this.canvas.height / rect.height
    }
    update(context){
        this.frame++
        const { ctx } = context.fetch(Renderer)
        const { x, y } = ctx.getTransform().inverse().transformPoint(this.position)
        this.x = x
        this.y = y
    }
}

class Keyboard {
    keys = Object.create(null)
    constructor(context){
        addEventListener('keydown', event => this.keys[event.key] = context.frame)
        addEventListener('keyup', event => this.keys[event.key] = undefined)
    }
    pressed(key){ return this.keys[key] >= 0 }
    update(context){}
}

function DomElement(tag, { style, ...props }, children){
    const element = document.createElement(tag)
    Object.assign(element.style, style)
    Object.assign(element, props)
    if(children) for(child of children) element.appendChild(child)
    return element
}