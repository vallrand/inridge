function shuffle(array){
    for (let i = array.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [array[i], array[j]] = [array[j], array[i]];
    }
    return array
}

class HexGrid {
    static center = new DOMPoint()
    static round(hex, out){
        let x = hex.x, y = hex.y
        const xgrid = Math.round(x), ygrid = Math.round(y)
        x -= xgrid, y -= ygrid
        out.x = xgrid + Math.round(x + 0.5*y) * (x*x >= y*y)
        out.y = ygrid + Math.round(y + 0.5*x) * (x*x < y*y)
        return out
    }
    hex2cart(hex, out){
        out.x = this.size * (3./2 * hex.x)
        out.y = this.size * (Math.sqrt(3)/2 * hex.x  +  Math.sqrt(3) * hex.y)
        return out
    }
    cart2hex(cart, out){
        out.x = ( 2./3 * cart.x) / this.size
        out.y = (-1./3 * cart.x  +  Math.sqrt(3)/3 * cart.y) / this.size
        return HexGrid.round(out, out)
    }

    cursor = { x: 0, y: 0 }
    constructor({ columns, rows, size, flat }){
        Object.assign(this, { padding: 4, columns, rows, size })
        this.coordinates = Array(this.columns * this.rows).fill(null).map((_,i) => ({ x: i % this.columns, y: i / this.columns | 0 }))
        this.grid = Array(this.columns * this.rows).fill(null)

        for(let r = 0; r < this.rows; r++) for(let c = 0; c < this.columns; c++){
            if(c + r < Math.floor(this.columns / 2)) this.coordinates[c + r * this.columns] = null
            else if(c + r >= this.columns + Math.floor(this.columns / 2)) this.coordinates[c + r * this.columns] = null
        }

        const directions = [[+1,0],[+1,-1],[0,-1],[-1,0],[-1,+1],[0,+1]]
        this.neighbors = this.coordinates.map(tile => !tile ? [] : directions.map(offset => {
            const nc = tile.x + offset[0], nr = tile.y + offset[1]
            if(nc < 0 || nr < 0 || nc >= this.columns || nr >= this.rows) return -1
            return nc + nr * this.columns
        }).filter(i => i != -1 && this.coordinates[i]))
    }
    index(c, r){
        if(c < 0 || r < 0 || c >= this.columns || r >= this.rows) return -1
        return this.coordinates[c + r * this.columns] ? c + r * this.columns : -1
    }
    spread(start, limit){
        const stack = [start], visited = [], out = []
        visited[start] = 1
        const agent = this.grid[start].agent
        while(stack.length){
            const index = stack.shift()
            if(!this.grid[index] || this.grid[index].agent !== agent) continue
            out.push(index)
            if(!this.grid[index].active) continue
            if(limit && visited[index] > limit) continue
            for(let neighbor of this.neighbors[index])
                if(!visited[neighbor]){
                    visited[neighbor] = visited[index] + 1
                    stack.push(neighbor)
                }
        }
        return out
    }
    group(start, agent){
        const stack = [], out = [], visited = []
        visited[start] = true
        stack.push(start)
        while(stack.length){
            const index = stack.pop()
            // if(index === start || !this.grid[index])

            if(this.grid[index] && this.grid[index].agent === agent && this.grid[index].active) out.push(index)
            if(index !== start && !this.grid[index]) continue
            for(let neighbor of this.neighbors[index])
                if(!visited[neighbor]){
                    visited[neighbor] = true
                    stack.push(neighbor)
                }
        }
        return out
    }
    render(ctx){
        for(let q = 0; q < this.columns; q++) for(let r = 0; r < this.rows; r++){
            if(!this.coordinates[q + r * this.columns]) continue
            const hover = this.cursor.x == q && this.cursor.y == r
            
            ctx.save()
            const center = this.hex2cart({ x: q, y: r }, HexGrid.center)
            ctx.translate(center.x, center.y)
            this.renderHex(ctx, 0, 0)

            ctx.fillStyle = hover ? 'hsla(108,62%,53%,1)' : 'hsla(210,27%,16%,1)'
            ctx.fill()

            const cell = this.grid[q + r * this.columns]
            if(cell) cell.render(ctx, this)
            ctx.restore()
        }
    }
    renderHex(ctx, centerX, centerY){
        ctx.beginPath()
        for(let i = 0; i < 6; i++){
            const x = centerX + (this.size - this.padding) * Math.cos(i * Math.PI / 3)
            const y = centerY + (this.size - this.padding) * Math.sin(i * Math.PI / 3)
            if(!i) ctx.moveTo(x, y)
            else ctx.lineTo(x, y)
        }
        ctx.closePath()
    }
}

class Reticle {
    radius = 200
    x = 0
    y = 0
    render(ctx){
        ctx.beginPath()
        ctx.moveTo(this.x - this.radius, this.y)
        ctx.lineTo(this.x + this.radius, this.y)
        ctx.moveTo(this.x, this.y - this.radius)
        ctx.lineTo(this.x, this.y + this.radius)
        ctx.lineWidth = 1
        ctx.strokeStyle = '#ffffff'
        ctx.stroke()
    }
}

const remap = (min0, max0, min1, max1, value) =>
    min1 + (max1 - min1) * ((value - min0) / (max0 - min0))
const lerp = (min, max, t) => min + t * (max - min)