import sqlite3
from math import pi
from collections import Counter

import pandas as pd

from bokeh.palettes import Category20c
from bokeh.plotting import figure, show,save
from bokeh.transform import cumsum
# db ops class
class DB(object):
    def __init__(self,dbpath: str):
        self.conn=sqlite3.connect(dbpath)
    def last_date(self):
        '''query the last date '''
        sql="SELECT date_ FROM everydaytask ORDER BY id DESC"
        cursor=self.conn.cursor()
        cursor.execute(sql)
        ret=cursor.fetchone()
        self.conn.commit()
        # cursor.close()
        return ret
    def latest_data(self):
        '''query and return records of the latest date.'''
        date=self.last_date()
        sql=f"SELECT begin_ts,end_ts,one_task_dur,task,detail FROM everydaytask WHERE date_='{date[0]}'"
        cursor=self.conn.cursor()
        cursor.execute(sql)
        # self.conn.commit()
        ret=cursor.fetchall()
        # cursor.close()
        return ret

class Data(object):
    '''use law of set(e.g. a^b,a and b...) to handle repeat elements'''
    def __init__(self,l: list):
      self.raw=l
    def tasks(self):
        '''task name as labels of pie chart'''
        return [i[3] for i in self.raw]
    def dur(self):
        '''duration time of every task (min)'''
        return [i[2] for i in self.raw]   
class ProcessData(object):
    def __init__(self,k: list[str],v: list[int]) :
        self.k=k
        self.v=v
    def index(self,word: str,l: list)-> tuple:
        '''
        l=['我','你','我','我','你']

        word='我'

        output:
        (0, 2, 3)
        '''
        n=0
        t=[]
        for a in l:
            if word==a:
                t.append(n)
            n+=1
        return tuple(t)

    def get_value_by_index(self,index: tuple,l: list[int])-> int:
        '''
        index=(1,4)

        l=[1,2,3,4,5]

        output:

        v1=l[1]=2

        v2=l[4]=5

        v1+v2=7
        '''
        s=0

        for i in index:
            s+=l[i] 
        return s
    def process_data(self):
        ke=self.k
        va=self.v
        c=Counter(ke)
        more_one=[]
        d={}
        dl={}
        for (w,t) in c.items():
        # try remove this,counter==1 is also ok
        # if t>1:
            more_one.append(w)
        for i in more_one:
            t=self.index(i,ke)
            d[i]=t

        for k,v in d.items():
            s=self.get_value_by_index(v,va)
            dl[k]=s
        return dl
# draw pie chart class
class Pie(Data):
    def __init__(self,db_records: list):
      super().__init__(db_records)  
    #   self.records=db_records
    def process_data(self):
        key=self.tasks()
        value=self.dur()
        return ProcessData(key,value).process_data()
    def plot(self):
        x=self.process_data()
        data = pd.Series(x).reset_index(name='value').rename(columns={'index': 'country'})
        data['angle'] = data['value']/data['value'].sum() * 2*pi
        data['color'] = Category20c[len(x)]

        p = figure(height=350, title="Pie Chart", toolbar_location=None,
           tools="hover", tooltips="@country: @value", x_range=(-0.5, 1.0))

        p.wedge(x=0, y=1, radius=0.4,
        start_angle=cumsum('angle', include_zero=True), end_angle=cumsum('angle'),
        line_color="white", fill_color='color', legend_field='country', source=data)

        p.axis.axis_label = None
        p.axis.visible = False
        p.grid.grid_line_color = None
        # show(p)
        save(p,'../storage/shared/plot.html')
        
db_path='task.db'
db=DB(db_path)
d=db.latest_data()

p=Pie(d)
p.plot()


