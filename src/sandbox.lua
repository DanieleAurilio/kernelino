-- Runs in every new process before the script.
-- Hides what could reach the host and makes preemption invisible to scripts.
local write, take_preempted, preempt, killed = ...

local raw_create, raw_resume, raw_yield = coroutine.create, coroutine.resume, coroutine.yield
local isyieldable, status = coroutine.isyieldable, coroutine.status
local raw_pcall, raw_xpcall, raw_load = pcall, xpcall, load
local pack, unpack, concat = table.pack, table.unpack, table.concat
local select, tostring, error = select, tostring, error

-- A killed process must not be able to catch its own death.
local function checked(results)
  local reason = killed()
  if reason then
    error(reason, 0)
  end
  return unpack(results, 1, results.n)
end

-- A preemption inside a coroutine yields to the script, not to the scheduler:
-- pass it up, then resume the coroutine as if nothing happened.
function coroutine.resume(co, ...)
  local results = pack(raw_resume(co, ...))
  while take_preempted() and status(co) == "suspended" do
    if isyieldable() then
      preempt()
      raw_yield()
    end
    results = pack(raw_resume(co))
  end
  return checked(results)
end

function coroutine.yield(...)
  -- A preemption refused in a non-yieldable section must not be mistaken for this yield.
  take_preempted()
  return raw_yield(...)
end

function coroutine.wrap(f)
  local co = raw_create(f)
  return function(...)
    local results = pack(coroutine.resume(co, ...))
    if not results[1] then
      error(results[2], 0)
    end
    return unpack(results, 2, results.n)
  end
end

function pcall(...)
  return checked(pack(raw_pcall(...)))
end

function xpcall(...)
  return checked(pack(raw_xpcall(...)))
end

-- Text chunks only: crafted bytecode can corrupt the VM.
function load(chunk, name, _, env)
  return raw_load(chunk, name, "t", env)
end

function print(...)
  local parts = {}
  for i = 1, select("#", ...) do
    parts[i] = tostring((select(i, ...)))
  end
  write(concat(parts, "\t") .. "\n")
end

dofile, loadfile, string.dump = nil, nil, nil
