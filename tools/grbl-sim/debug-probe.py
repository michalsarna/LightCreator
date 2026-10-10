# DEBUG (to be removed): time grbl-sim on the CI runner.
import socket,time,sys,subprocess,os
here=os.path.dirname(os.path.abspath(__file__))
job=[l for l in open(os.path.join(here,"debug-job.nc")).read().split("\n") if l.strip()][:185]
srv=subprocess.Popen([sys.executable,os.path.join(here,"serve.py"),"--port","0"],stdout=subprocess.PIPE,text=True)
port=int(srv.stdout.readline().strip().rsplit(":",1)[1])
def run(settings, job, poll, limit=90):
    s=socket.create_connection(("127.0.0.1",port)); s.settimeout(0.002); buf=[b""]
    def lines():
        try: buf[0]+=s.recv(4096)
        except socket.timeout: pass
        out=[]
        while b"\n" in buf[0]:
            l,buf[0]=buf[0].split(b"\n",1); l=l.strip().decode()
            if l: out.append(l)
        return out
    t=time.time(); s.send(b"\x18")
    while not any(l.startswith("Grbl") for l in lines()):
        if time.time()-t>20: return "no banner"
    tb=time.time()-t
    def stream(job):
        pending=[];i=0;done=0;t=time.time();last=0;st=""
        while done<len(job) and time.time()-t<limit:
            for l in lines():
                if l=="ok" or l.startswith("error"): pending.pop(0); done+=1
                elif l.startswith("<"): st=l
            while i<len(job) and sum(pending)+len(job[i])+1<=120:
                s.send((job[i]+"\n").encode()); pending.append(len(job[i])+1); i+=1
            if poll and time.time()-last>poll: s.send(b"?"); last=time.time()
        return "%d/%d in %.1fs %s"%(done,len(job),time.time()-t,st[:45])
    stream(settings); r="banner %.2fs, %s"%(tb,stream(job)); s.close(); return r
fast=["$100=20","$101=20","$110=30000","$111=30000","$120=5000","$121=5000","$32=1"]
slow250=["$110=30000","$111=30000","$120=5000","$121=5000","$32=1"]
move=["G21","G90","G1 X10 F600","G4 P0.01"]
for name,st,jb,poll in [("move default",[],move,0.25),("dense fast poll",fast,job,0.25),("dense fast nopoll",fast,job,0),("dense 250steps poll",slow250,job,0.25)]:
    t=time.time(); r=run(st,jb,poll)
    print("::notice title=probe %s::%s"%(name,r), flush=True)
print("::notice title=cpus::%s"%os.cpu_count())
srv.kill()
