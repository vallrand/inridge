class Construction {
    static gather(map, network){
        const queue = []
        for(let i = 0; i < map.grid.length; i++) if(map.grid[i] && map.grid[i].loading) queue.push(i)
        queue.sort((a,b) => map.grid[a].frame - map.grid[b].frame)
        for(let i of queue){
            const cell = map.grid[i]
            
            const adjacent = map.neighbors[i].map(index => network.visited[index])
            .filter((e,i,a) => e > 0 && a.indexOf(e) === i)
            .map(group => network.groups[group - 1])
            let matter = adjacent.reduce((total, { matter }) => total + matter, 0)
            if(!network.groups.length && i === queue[0]) matter = Math.max(matter, 1)

            if(cell.suspended) continue
            let consumed = 0, initial = matter
            if(initial < cell.loading.cost) continue
            matter -= cell.loading.cost
            consumed += cell.loading.cost
            cell.loading.value += cell.loading.cost

            //consume from resource?
            for(let remaining = consumed; remaining > 0; remaining--){
                for(let group of adjacent) if(group.matter > 0){
                    group.matter--
                    group.consumption++
                    break
                }
            }
        }
    }
    static update(map, network){
        for(let i = 0; i < map.grid.length; i++) if(map.grid[i]){
            const cell = map.grid[i]
            if(cell.loading && cell.loading.value >= cell.loading.max){
                cell.loading = null
                if(cell.health) cell.health.value = cell.health.max
            }
        }
    }
    constructor(max, upgrade){Object.assign(this, { max, upgrade })}
    get progress(){ return this.value / this.max }
    value = 0
    multiplier = 0
    cost = 1
    active = false
    toString(){return `Under construction ${this.value}/${this.max}`}
}

class Suspended {
    toString(){return `Suspended`}
}

class Integrity {
    static fragment_exchange_rate = 10
    static update(map, network){
        destroy: for(let i = 0; i < map.grid.length; i++) if(map.grid[i]){
            const cell = map.grid[i]
            if(cell.health && cell.health.value < 0)
                map.grid[i] = null
            else if(cell.active && cell.health && cell.health.value < cell.health.max){
                while(cell.health.fragments > Integrity.fragment_exchange_rate){
                    cell.health.value = Math.min(cell.health.value + 1, cell.health.max)
                    cell.health.fragments -= Integrity.fragment_exchange_rate
                }
            }
        }
    }
    constructor(max){this.max = max}
    value = 0
    armor = 0
    remainder = 0
    fragments = 0
    damage(delta, frame){
        const amortization = this.armor + 1
        this.remainder = this.remainder % amortization
        const d = (delta + this.remainder) / amortization | 0
        this.remainder = (delta + this.remainder) % amortization
        delta = d        

        this.value -= delta
        this.last = frame
    }
    toString(){return `Integrity ${this.value}/${this.max} restore ${this.fragments}/${Integrity.fragment_exchange_rate} armor ${this.armor}`}
}

class IntegrityReconstruction {
    static gather(map, network){
        restore: for(let group of network.groups)
            for(let i of group.list){
                const cell = map.grid[i]
                if(!cell.restore) continue
                cell.restore.count = 0
                
                const affected = cell._affected = map.spread(i, cell.restore.range)
                affected.sort((a,b) => map.grid[a].frame - map.grid[b].frame)

                for(let j of affected){
                    const target = map.grid[j]
                    if(target.active && target.health && target.health.value < target.health.max){
                        const cost = 1
                        cell.group.consumption += cost
                        if(cell.group.matter < cost) continue
                        cell.group.matter -= cost
                        target.health.fragments += cost

                        cell.restore.count++
                        if(cell.restore.count >= (cell.restore.multiplier + 1)) break
                    }
                }
            }
    }
    count = 0
    get active(){ return this.count > 0 }
    radius = 1
    extend = 0
    multiplier = 0
    extra = 0
    get range(){ return this.radius + this.extend }
    toString(){return `restoring ${this.count}`}
}

class MatterProduction {
    static update(map, network){
        for(let group of network.groups) group.matter = 0
        for(let cell of map.grid) if(cell && !cell.loading && !cell.suspended && cell.matter){
            const delta = (cell.matter.add + cell.matter.extra) * (1 + cell.matter.multiplier)
            cell.group.matter += delta
        }
        for(let group of network.groups) group.production = group.matter
    }
    constructor(initial = 1){this.add = initial}
    extra = 0
    multiplier = 0
    active = true
    get total(){ return (this.add + this.extra) * (1 + this.multiplier) }
    toString(){return `Generate (${this.add} + ${this.extra}) * ${this.multiplier+1} = <b>${this.total}</b> matter per step`}
}

class MatterCollection {
    static update(map, network){
        unload_carry_over: for(let cell of map.grid) if(cell && !cell.loading && !cell.suspended && cell.store){
            cell.group.matter += cell.store.value
            cell.store.persist = cell.store.value
            cell.store.value = 0
        }
    }
    static gather(map, network){
        load: for(let group of network.groups){
            group.stored = 0
            let overflow = group.production - group.consumption
            for(let i of group.list){
                const cell = map.grid[i]
                if(!cell || cell.loading || cell.suspended || !cell.store) continue

                const recover = Math.min(cell.store.max, cell.store.persist, group.matter)
                const collect = Math.min(cell.store.cap + cell.store.multiplier, group.matter - recover, Math.max(0, overflow), cell.store.max - recover)
                const transfer = recover + collect
                overflow -= collect
                cell.group.matter -= transfer
                cell.store.value = Math.min(cell.store.max, cell.store.value + recover + collect * (cell.store.extra + 1))
                group.stored += cell.store.value
            }
        }
    }
    persist = 0
    value = 0
    cap = 1
    multiplier = 0
    extra = 0
    active = false
    constructor(initial = 10){this.max = initial}
    toString(){return `Storage ${this.value}/${this.max} matter (throughput ${this.cap})`}
}

class MatterConsumption {
    static gather(map, network){
        load: for(let group of network.groups)
            for(let i of group.list){
                const cell = map.grid[i]
                if(!cell.consume) continue

                reset: cell.consume.active = false
                
                if(cell.loading || cell.suspended || !cell.consume) continue
                cell.group.consumption += cell.consume.total
                if(cell.group.matter < cell.consume.total){
                    cell.group.matter = 0
                    continue
                }
                cell.group.matter -= cell.consume.total
                cell.consume.active = true
            }
    }
    active = false
    multiplier = 0
    discount = 0
    get total(){ return Math.max(1, this.amount * (this.multiplier + 1) - this.discount) }
    constructor(initial = 1){this.amount = initial}
    toString(){return `Require ${this.amount} * ${1 + this.multiplier} - ${this.discount} = <b>${this.total}</b> ${this.active ? '<b>active</b>' : 'inactive'}`}
}

class PropagateModifier {
    static radius = {apply(cell, modifier){ //as component with reset/equation in target affected component?
        if(cell.modify) cell.modify.extend += modifier._delta
        if(cell.targeting) cell.targeting.extend += modifier._delta
        if(cell.restore) cell.restore.extend += modifier._delta
        if(cell.production) cell.production.extend += modifier._delta
    },toString(){return 'radius of effect'}}
    static efficiency = {apply(cell, modifier){
        if(cell.matter) cell.matter.extra += modifier._delta
        if(cell.targeting) cell.targeting.extra += modifier._delta
        if(cell.restore) cell.restore.extra += modifier._delta
        if(cell.production) cell.production.extra += modifier._delta
        if(cell.modify && (cell.modify.modifier === PropagateModifier.cost || cell.modify.modifier === PropagateModifier.armor)) cell.modify.extra += modifier._delta

    },toString(){return 'efficiency'}}
    static rate = {apply(cell, modifier){
        if(cell.matter) cell.matter.multiplier += modifier._delta
        if(cell.loading) cell.loading.multiplier += modifier._delta
        if(cell.collect) cell.collect.multiplier += modifier._delta
        if(cell.targeting) cell.targeting.multiplier += modifier._delta
        if(cell.production) cell.production.multiplier += modifier._delta
        if(cell.restore) cell.restore.multiplier += modifier._delta
        if(cell.consume && !cell.modify) cell.consume.multiplier += modifier._delta
    },toString(){return 'speed'}}
    static cost = {apply(cell, modifier){
        if(cell.consume) cell.consume.discount += modifier._delta
    },toString(){return 'cost reduction / discount'}}
    static armor = {apply(cell, modifier){
        if(cell.health) cell.health.armor += modifier._delta
        if(cell.production) cell.production.armor += modifier._delta
    },toString(){return 'integrity/armor'}}
    static release(map, network){
        precalculate: for(let i = 0; i < map.grid.length; i++) if(map.grid[i]){
            const cell = map.grid[i]
            if(cell.active && cell.modify && cell.consume && cell.consume.active){
                cell.modify._radius = cell.modify.radius + cell.modify.extend
                cell.modify._delta = cell.modify.delta + cell.modify.extra
            }
            if(cell.health) cell.health.armor = 0
            if(cell.restore) cell.restore.multiplier = cell.restore.extra = cell.restore.extend = 0
            if(cell.matter) cell.matter.multiplier = cell.matter.extra = 0
            if(cell.loading) cell.loading.multiplier = 0
            if(cell.collect) cell.collect.multiplier = cell.collect.extra = 0
            if(cell.targeting) cell.targeting.multiplier = cell.targeting.extra = cell.targeting.extend = 0
            if(cell.production) cell.production.multiplier = cell.production.extend = cell.production.extra = cell.production.armor = 0
            if(cell.consume) cell.consume.multiplier = cell.consume.discount = 0
            if(cell.modify) cell.modify.extend = cell.modify.extra = 0

        }
        apply_effects: for(let i = 0; i < map.grid.length; i++) if(map.grid[i] && map.grid[i].active)
        if(map.grid[i].modify && map.grid[i].consume && map.grid[i].consume.active){
            const cell = map.grid[i]
            const affected = map.spread(i, cell.modify._radius)
            cell._affected = affected
            for(let j = 1; j < affected.length; j++){
                const target = map.grid[affected[j]]
                cell.modify.modifier.apply(target, cell.modify)
            }
        }
    }
    constructor(modifier, delta){Object.assign(this, { modifier, delta })}
    radius = 1
    extend = 0
    extra = 0
    toString(){return `propagate ${this.delta} ${this.modifier} by ${this._radius} cells`}
}

class UnitProduction {
    static update(map, network){
        for(let i = 0; i < map.grid.length; i++) if(map.grid[i]){
            const cell = map.grid[i]
            if(cell.suspended || cell.loading || !cell.production || !cell.consume.active) continue
            cell.production.stack += cell.production.multiplier + 1
            while(cell.production.stack > cell.production.max){
                cell.production.stack -= cell.production.max


                const center = map.hex2cart(map.coordinates[i], { x: 0, y: 0 })
                for(let amount = cell.production.multiplier + 1; amount > 0; amount--){
                    const unit = new LocustUnit(center.x, center.y)
                    unit.agent = network.agent
                    unit.damage += cell.production.extra
                    unit.velocity += 10 * 0.001 * cell.production.extend
                    unit.armor += cell.production.armor
                    network.parent.units.push(unit)
                }
            }

        }
    }
}

class LocustUnit {
    constructor(x, y){
        this.x = x
        this.y = y
        this.vx = this.vy = 0
    }
    agent = 2
    damage = 1
    health = 1
    armor = 0
    size = 20
    velocity = 0.001 * 20
    applyDamage(delta, frame){
        this.health -= delta
    }
    update({ map, units }, { deltaTime, frame }){
        if(this.health <= 0) return false
        this.target = this.findTarget(map, units)
        if(!this.target) return this.randomWalk(deltaTime)
        this.vx = this.target.x - this.x
        this.vy = this.target.y - this.y
        const distance = Math.sqrt(this.vx*this.vx+this.vy*this.vy)
        if(distance < this.size && this.target.index != null){
            map.grid[this.target.index].health.damage(this.damage, frame)
            return false
        }else if(distance < this.size){
            this.target.applyDamage(this.damage, frame)
            return false
        }else{
            this.x += this.velocity * deltaTime * this.vx / distance
            this.y += this.velocity * deltaTime * this.vy / distance
        }
    }
    randomWalk(deltaTime){
        const angle = Math.atan2(this.vy, this.vx) + deltaTime * 0.001 * (-0.5+Math.random()) * 4 * Math.PI
        this.vx = Math.cos(angle) * this.velocity
        this.vy = Math.sin(angle) * this.velocity
        this.x += deltaTime * this.vx
        this.y += deltaTime * this.vy
    }
    findTarget(map, units){
        let min = Infinity, target = null
        for(let unit of units) if(unit instanceof LocustUnit && unit.agent !== this.agent){
            const dx = this.x - unit.x, dy = this.y - unit.y
            const distance = dx*dx+dy*dy
            if(min <= distance) continue
            min = distance
            target = unit
        }
        for(let i = 0; i < map.grid.length; i++){
            if(!map.grid[i] || map.grid[i].agent === this.agent || map.grid[i].health.value < 0) continue
            const center = map.hex2cart(map.coordinates[i], {x:0,y:0})
            const dx = this.x - center.x, dy = this.y - center.y
            const distance = dx*dx+dy*dy
            if(min <= distance) continue
            min = distance
            target = { x: center.x, y: center.y, index: i }
        }
        return target

        // if(this.target && map.grid[this.target.index] && map.grid[this.target.index].health.value >= 0) return this.target
        // const candidates = []
        // for(let i = 0; i < map.grid.length; i++){
        //     if(!map.grid[i] || map.grid[i].agent === this.agent || map.grid[i].health.value < 0) continue
        //     const center = map.hex2cart(map.coordinates[i], {x:0,y:0})
        //     candidates.push({ x: center.x, y: center.y, index: i })
        // }
        // return candidates[Math.random() * candidates.length | 0]
    }
    render(ctx){
        ctx.save()
        ctx.translate(this.x, this.y)
        ctx.rotate(Math.atan2(this.vy, this.vx))
        ctx.fillStyle = this.agent === 1 ? '#34eb9b' : '#c04020'
        ctx.textAlign = 'center'
        ctx.textBaseline = 'middle'
        ctx.font = `${this.size}px Black Arial`
        ctx.fillText('➤', 0, 0)
        ctx.restore()
    }
}
class Projectile {
    damage = 1
    agent = 1
    constructor(start, end){
        this.start = start
        this.end = end
        this.progress = 0
        this.duration = 0.4
    }
    update(parent, context){
        this.progress += context.deltaTime * 0.001 / this.duration
        if(this.progress > 1){
            this.end.health -= this.damage
            return false
        }
    }
    render(ctx){
        const f1 = Math.min(2 * this.progress, 1)
        const f0 = Math.max(0, 2 * this.progress - 1)
        const x0 = lerp(this.start.x, this.end.x, f0)
        const y0 = lerp(this.start.y, this.end.y, f0)
        const x1 = lerp(this.start.x, this.end.x, f1)
        const y1 = lerp(this.start.y, this.end.y, f1)

        ctx.beginPath()
        ctx.moveTo(x0, y0)
        ctx.lineTo(x1, y1)
        ctx.lineWidth = 4

        const gradient = ctx.createLinearGradient(x0, y1, x1, y1)
        gradient.addColorStop(0, 'rgba(255,0,0,0)')
        gradient.addColorStop(1, 'rgba(255,255,255,1)')
        ctx.strokeStyle = gradient
        ctx.stroke()
    }
}