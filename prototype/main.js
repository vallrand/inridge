class Tile {
    renderIcon(ctx, parent){
        ctx.save()
        ctx.fillStyle = this.suspended ? '#aaaaaa' : this.constructor.color
        ctx.textAlign = 'center'
        ctx.textBaseline = 'middle'
        ctx.font = `${parent.size}px Black Arial`
        ctx.fillText(this.constructor.icon, 0, 0)
        if(this.active && this.consume && this.consume.active){
            ctx.strokeStyle = '#00ffff'
            ctx.lineWidth = 1
            ctx.strokeText(this.constructor.icon, 0, 0)
        }
        ctx.restore()
    }
    render(ctx, parent){
        this.renderIcon(ctx, parent)

        disabled: if(!this.active || this.consume && !this.consume.active){
            ctx.fillStyle = this.suspended ? '#888888' : '#ffff00'
            ctx.textAlign = 'center'
            ctx.textBaseline = 'middle'
            ctx.font = `${parent.size}px Black Arial`
            ctx.fillText('⊘'||`⊗⊘⍉`, 0, 0)
        }

        upgrade: if(this.loading){
            ctx.beginPath()
            ctx.arc(0, 0, parent.size * 0.8, 0, 2 * Math.PI - this.loading.progress * 2 * Math.PI, false)
            ctx.arc(0, 0, parent.size * 0.4, 2 * Math.PI - this.loading.progress * 2 * Math.PI, 0, true)
            ctx.closePath()
            ctx.fillStyle = '#ffffff'
            ctx.fill()
            return
        }

        health: if(this.health){
            ctx.lineWidth = 1
            ctx.strokeStyle = '#e0e0e0'
            ctx.fillStyle = '#ff0000'
            ctx.fillRect(-parent.size * 0.5, 0.5 * parent.size, parent.size * Math.max(0,this.health.value) / this.health.max, 8)
            ctx.strokeRect(-parent.size * 0.5, 0.5 * parent.size, parent.size, 8)
        }

        for(let mods = this.mods, i = 0; i < mods.length; i++){
            const x = (parent.size/2) * Math.cos(i * Math.PI / 4)
            const y = (parent.size/2) * Math.sin(i * Math.PI / 4)

            ctx.font = `${parent.size*0.8|0}px Arial Black`
            ctx.fillStyle = mods[i].color
            ctx.fillText(`+`, x, y)
        }
    }
    get mods(){
        const out = []
        for(let key in this) if(this[key] && this[key].extra){
            out.push({ color: '#d98218' })
            break
        }
        for(let key in this) if(this[key] && this[key].multiplier){
            out.push({ color: '#42f5b6' })
            break
        }
        for(let key in this) if(this[key] && this[key].extend){
            out.push({ color: '#99f542' })
            break
        }
        if(this.health && this.health.armor) out.push({ color: '#5229ab' })
        
        // if(this.matter && this.matter.extra) out.push({ color: '#d98218' })
        // if(this.matter && this.matter.multiplier) out.push({ color: '#42f5b6' })
        return out
    }
    get active(){ return !this.loading }
}

class NetworkGroup {
    groups = []
    visited = []
    count = 0
    constructor(map, agent, parent){
        this.parent = parent
        this.map = map
        this.agent = agent
        this.grid = Array(this.map.grid.length).fill(null)
    }
    adjacent(index){
        if(index < 0) return []
        if(this.map.grid[index] && this.map.grid[index].active && this.visited[index]) return [this.groups[this.visited[index] - 1]]
        return this.map.neighbors[index].map(i => this.visited[i])
        .filter((e,i,a)=> e > 0 && a.indexOf(e) === i)
        .map(group => this.groups[group - 1])
    }
    update({ map, units }){
        Integrity.update(map, this)
        Construction.update(map, this)

        this.count = this.visited.length = this.groups.length = 0
        link: for(let i = 0; i < map.grid.length; i++) if(map.grid[i] && map.grid[i].active && !this.visited[i]){
            if(map.grid[i].agent !== this.agent) continue
            const group = map.group(i, map.grid[i].agent)
            group.sort((a,b) => map.grid[a].frame - map.grid[b].frame)
            const summary = {
                list: group,
                cells: group.length, matter: 0, consumption: 0, production: 0
            }
            const index = this.groups.push(summary)
            for(let cell of group){
                map.grid[cell].group = summary //group is component
                this.visited[cell] = index
            }
        }

        MatterProduction.update(map, this)
        MatterCollection.update(map, this)

        allocate_resources: {
            MatterConsumption.gather(map, this)
            IntegrityReconstruction.gather(map, this)
            Construction.gather(map, this)
            MatterCollection.gather(map, this)
        }

        PropagateModifier.release(map, this)
        UnitProduction.update(map, this)
    }
}

class ConstructionSite extends Tile {
    constructor(prev, next){
        this.delegate = next
        this.loading = new Construction(10)
    }
    render(ctx, parent){
        this.delegate.render(ctx, parent)
    }
    get ative(){}
}

const buildings = [
    class Extractor extends Tile {
        static icon = '✱'
        static color = '#d9d618'
        static description = `<i>Extract materials from beneath the crust. Generates resources every step</i>`
        static validate = (cell, index, network) => cell === null && (network.groups.length ? network.adjacent(index).length : true)

        loading = new Construction(10)
        health = new Integrity(5)
        matter = new MatterProduction(1)
        toString(){return [this.constructor.name,this.loading || '',this.health,
            this.matter,
            this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class Capacitor extends Tile {
        static icon = '🞑'
        static color = '#10e8a7'
        static description = `<i>Collects and stores overflow resources.</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(10)
        health = new Integrity(5)
        store = new MatterCollection(10)
        toString(){return [
            this.constructor.name,this.loading || '',this.health,this.store,this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class CapacitorTierII extends Tile {
        static icon = '|🞑|'
        static color = '#10e8a7'
        static description = `<i>Collects and stores overflow resources.</i>`
        static validate = (cell, index, network) => cell && cell.agent === network.agent && cell.constructor.name === 'Capacitor' && cell.active

        loading = new Construction(20, true)
        health = new Integrity(10)
        store = new MatterCollection(25)
        toString(){return [
            this.constructor.name,this.loading || '',this.health,this.store,this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class Amplifier extends Tile {
        static icon = '◉'
        static color = '#99f542'
        static description = `<i>Increase range/radius</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(10)
        health = new Integrity(5)
        consume = new MatterConsumption(2)
        modify = new PropagateModifier(PropagateModifier.radius, 1)
        toString(){return [
            this.constructor.name,this.loading || '',this.health,this.consume,this.modify,this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class Accelerator extends Tile {
        static icon = '✪'
        static color = '#42f5b6'
        static description = `<i>Increases construction/production/collection/restoration rate to adjacent modules</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(10)
        health = new Integrity(5)
        consume = new MatterConsumption(2)
        modify = new PropagateModifier(PropagateModifier.rate, 1)
        toString(){return [
            this.constructor.name,this.loading || '',this.health,this.consume,this.modify,this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class MagnifierCatalystConcentrator extends Tile {
        static icon = '✜'
        static color = '#d98218'
        static description = `<i>Increase throughput/efficiency to adjacent</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(10)
        health = new Integrity(5)
        consume = new MatterConsumption(2)
        modify = new PropagateModifier(PropagateModifier.efficiency, 1)
        toString(){return [
            this.constructor.name,this.loading || '',this.health,this.consume,this.modify,this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class Converter extends Tile {
        static icon = '❖'
        static color = '#7d75bf'
        static description = `<i>Reduces consumption</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(10)
        health = new Integrity(5)
        consume = new MatterConsumption(1)
        modify = new PropagateModifier(PropagateModifier.cost, 1)
        toString(){return [
            this.constructor.name,this.loading || '',this.health,this.consume,this.modify,this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class ReinforcerSolidifier extends Tile {
        static icon = '▲'
        static color = '#5229ab'
        static description = `<i>increase integrity and defence distributer compressor? increase density</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(15)
        health = new Integrity(20)
        consume = new MatterConsumption(1)
        modify = new PropagateModifier(PropagateModifier.armor, 1)
        toString(){return [
            this.constructor.name,this.loading || '',this.health,this.consume,this.modify,this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class Regenerator extends Tile {
        static icon = '❉'
        static color = '#7d75bf'
        static description = `<i>Restore integrity</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(15)
        health = new Integrity(5)
        restore = new IntegrityReconstruction()
        // consume = new MatterConsumption(1)
        // modify = new PropagateModifier(PropagateModifier.regeneration, 1)
        renderIcon(ctx, parent){
            ctx.save()
            if(this.active && this.restore && this.restore.active) ctx.rotate(Math.PI * performance.now() / 1000)
            super.renderIcon(ctx, parent)
            ctx.restore()
        }
        toString(){return [this.constructor.name,this.loading || '',this.health,this.restore,
            this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class Factory extends Tile {
        static icon = '▩'
        static color = '#7d75bf'
        static description = `<i>produce units</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(20)
        health = new Integrity(10)
        consume = new MatterConsumption(4)
        production = { multiplier: 0, extend: 0, extra: 0, armor: 0, stack: 0, max: 10, toString(){
            return `build ${this.multiplier} with ${this.amount} hp ${this.extra} damage ${this.radius} range/speed?`
        } }
        update(){}
        toString(){return [this.constructor.name,this.loading || '',this.health,this.consume,this.production,
            this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    class Turret extends Tile {
        static icon = '⚴'
        static color = '#991c52'
        static description = `<i>Defence system</i>`
        static validate = (cell, index, network) => cell === null && network.adjacent(index).length

        loading = new Construction(20)
        health = new Integrity(10)
        consume = new MatterConsumption(3)
        targeting = { multiplier: 0, radius: 100, extend: 0, extra: 0, damage: 1, rate: 1, rotation: 0, toString(){
            return `shoot in ${this.radius}+${this.extend} radius with ${this.damage}+${this.extra} damage`
        }, get range(){ return this.radius + this.extend * 50 } }
        elapsed = 0
        update(index, { map, units }, context){
            this.elapsed += 0.001 * context.deltaTime
            const cooldown = 1 / (this.targeting.rate + this.targeting.multiplier)
            if(this.elapsed < cooldown) return
            this.elapsed -= cooldown


            if(!this.consume.active) return
            const center = map.hex2cart(map.coordinates[index], {x:0,y:0})
            let closest = -1, min = this.targeting.range
            for(let i = units.length - 1; i >= 0; i--){
                const unit = units[i]
                if(unit.agent === this.agent || !unit.health) continue
                const distance = Math.hypot(unit.x - center.x, unit.y - center.y)
                if(distance >= min) continue
                min = distance
                closest = i 
            }
            if(closest != -1){
                const unit = units[closest]
                const dx = unit.x - center.x
                const dy = unit.y - center.y
                this.targeting.rotation = Math.atan2(dy, dx)
                units.push(new Projectile(center, unit))
            }
        }
        renderUI(ctx, center){
            if(this.suspended || this.loading) return
            ctx.beginPath()
            ctx.arc(center.x, center.y, this.targeting.range, 0, 2 * Math.PI, false)
            ctx.fillStyle = this.consume.active ? 'rgba(0,150,200,0.4)' : 'rgba(50,50,50,0.1)'
            ctx.fill()
        }
        renderIcon(ctx, parent){
            ctx.save()
            ctx.rotate(this.targeting.rotation + 0.5 * Math.PI)
            super.renderIcon(ctx, parent)
            ctx.restore()
        }
        toString(){return [this.constructor.name,this.loading || '',this.health,this.consume,this.targeting,
            this.constructor.description
        ].map(row => `<div>${row}</div>`).join('\n')}
    },
    //☬☫☤⚚⚶⚵

    class Portal extends Tile {
        static icon = '✺'
        static color = '#ff0000'
        static description = `<i>Enemy spawner</i>`
        static validate = (cell, index, network) => false
        toString(){return [this.constructor.name,this.constructor.description]}
        elapsed = 0
        health = Object.assign(new Integrity(10), { value: 10 })
        waves = []//[{ cooldown: 5, amount: 1 }, { cooldown: 10, amount: 5 }, { cooldown: 10, amount: 10 }]
        update(index, { map, units }, context){
            this.elapsed += 0.001 * context.deltaTime
            if(!this.waves.length || this.elapsed < this.waves[0].cooldown) return
            this.elapsed -= this.waves[0].cooldown
            const { amount } = this.waves.shift()
            const { x, y } = map.hex2cart(map.coordinates[index], { x: 0, y: 0 })
            for(let i = amount; i > 0; i--){
                units.push(new LocustUnit(x, y))
            }
        }
        start(){
            this.elapsed = 0
            this.waves = Array(20).fill().map((_,i) => ({ cooldown: 1 + i * 0.2, amount: 1 + i / 2 | 0 }))
        }
    }
]